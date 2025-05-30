.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl uservec
    .globl userret
    .align 2
uservec:
    csrrw sp, sscratch, sp
    # sp->*TrapFrame, sscratch -> user stack

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
    # load kernel satp into t0
    ld t0, 34*8(sp)
    # load trap_handler address into t1
    ld t1, 36*8(sp)
    # load kernel_sp to sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0 
    # flush TLB
    sfence.vma
    # jump to trap_handler
    jr t1

userret:
    # case1: start running app by userret
    # case2: back to U after handling trap
    
    # a0: *TrapFrame in user space
    # a1: user space token

    # switch to user space
    csrw satp, a1
    sfence.vma

    # save a0(*TrapFrame) to sscratch
    csrw sscratch, a0
    # sp points to TrapFrame in user space
    mv sp, a0
    
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    
    # restore general-purpose registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ld sp, 2*8(sp)  # restore user stack pointer
    sret
