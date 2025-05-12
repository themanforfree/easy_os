.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl uservec
    .globl userret
    .align 2
uservec:
    csrrw sp, sscratch, sp
    # 用户态陷入内核前，sscratch 保存着内核栈指针，发生陷入后
    # 通过以上指令换栈，然后再内核栈上分配 34*8 字节的空间作为 TrapFrame
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapFrame on kernel stack
    addi sp, sp, -34*8
    // 保存除了 x0/x2(sp)/x4(tp) 以外的所有通用寄存器
    // x0 是常数 0, 不需要保存
    // x2 是栈指针，目前指向内核栈，后面单独处理
    // x4 是线程指针，暂时不考虑
    # save general-purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    # 保存 sstatus/sepc
    # 保存 sstatus 是为了在返回用户态时，恢复 sstatus 的值，主要是特权级
    # sepc 保存的是发生异常的地址，返回用户态时需要恢复继续执行
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    # 读出用户栈指针，存入 TrapFrame
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # set input argument of trap_handler(cx: &mut TrapFrame)
    # 调用 rust 实现的 trap_handler，并传入 TrapFrame 的地址
    mv a0, sp
    call trap_handler

userret:
    # case1: start running app by userret
    # case2: back to U after handling trap
    mv sp, a0
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2 # 将用户栈指针恢复到 sscratch 寄存器
    # restore general-purpose registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapFrame on kernel stack
    addi sp, sp, 34*8 # 释放内核栈上的 TrapFrame
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp # 换栈
    sret
