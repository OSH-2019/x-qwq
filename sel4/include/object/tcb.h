/*
 * Copyright 2014, General Dynamics C4 Systems
 *
 * This software may be distributed and modified according to the terms of
 * the GNU General Public License version 2. Note that NO WARRANTY is provided.
 * See "LICENSE_GPLv2.txt" for details.
 *
 * @TAG(GD_GPL)
 */

#ifndef __OBJECT_TCB_H
#define __OBJECT_TCB_H

#include <types.h>
#include <api/failures.h>
#include <object/structures.h>

#include <machine/registerset.h>
#include <object/cnode.h>

#ifdef CONFIG_DEBUG_BUILD
/* Maximum length of the tcb name, including null terminator */
#define TCB_NAME_LENGTH (BIT(seL4_TCBBits) - TCB_OFFSET - sizeof(tcb_t))
compile_assert(tcb_name_fits, TCB_NAME_LENGTH > 0)
#endif

struct tcb_queue {
    tcb_t *head;
    tcb_t *end;
};
typedef struct tcb_queue tcb_queue_t;

static inline unsigned int
setMR(tcb_t *receiver, word_t* receiveIPCBuffer,
      unsigned int offset, word_t reg)
{
    if (offset >= n_msgRegisters) {
        if (receiveIPCBuffer) {
            receiveIPCBuffer[offset + 1] = reg;
            return offset + 1;
        } else {
            return n_msgRegisters;
        }
    } else {
        setRegister(receiver, msgRegisters[offset], reg);
        return offset + 1;
    }
}

void tcbSchedEnqueue(tcb_t *tcb);
void tcbSchedAppend(tcb_t *tcb);
void tcbSchedDequeue(tcb_t *tcb);

#ifdef CONFIG_DEBUG_BUILD
void tcbDebugAppend(tcb_t *tcb);
void tcbDebugRemove(tcb_t *tcb);
#endif

#ifdef ENABLE_SMP_SUPPORT
void remoteQueueUpdate(tcb_t *tcb);
void remoteTCBStall(tcb_t *tcb);

#define SCHED_ENQUEUE(_t) do {      \
    tcbSchedEnqueue(_t);            \
    remoteQueueUpdate(_t);          \
} while (0)

#define SCHED_APPEND(_t) do {       \
    tcbSchedAppend(_t);             \
    remoteQueueUpdate(_t);          \
} while (0)

#else
#define SCHED_ENQUEUE(_t)           tcbSchedEnqueue(_t)
#define SCHED_APPEND(_t)            tcbSchedAppend(_t)
#endif /* ENABLE_SMP_SUPPORT */

#define SCHED_ENQUEUE_CURRENT_TCB   tcbSchedEnqueue(NODE_STATE(ksCurThread))
#define SCHED_APPEND_CURRENT_TCB    tcbSchedAppend(NODE_STATE(ksCurThread))

tcb_queue_t tcbEPAppend(tcb_t *tcb, tcb_queue_t queue);
tcb_queue_t tcbEPDequeue(tcb_t *tcb, tcb_queue_t queue);

void setupCallerCap(tcb_t *sender, tcb_t *receiver);
void deleteCallerCap(tcb_t *receiver);

word_t copyMRs(tcb_t *sender, word_t *sendBuf, tcb_t *receiver,
               word_t *recvBuf, word_t n);
exception_t decodeTCBInvocation(word_t invLabel, word_t length, cap_t cap,
                                cte_t* slot, extra_caps_t excaps, bool_t call,
                                word_t *buffer);
exception_t decodeCopyRegisters(cap_t cap, word_t length,
                                extra_caps_t excaps, word_t *buffer);
exception_t decodeReadRegisters(cap_t cap, word_t length, bool_t call,
                                word_t *buffer);
exception_t decodeWriteRegisters(cap_t cap, word_t length, word_t *buffer);
exception_t decodeTCBConfigure(cap_t cap, word_t length,
                               cte_t* slot, extra_caps_t rootCaps, word_t *buffer);
exception_t decodeSetPriority(cap_t cap, word_t length, extra_caps_t excaps, word_t *buffer);
exception_t decodeSetMCPriority(cap_t cap, word_t length, extra_caps_t excaps, word_t *buffer);
exception_t decodeSetSchedParams(cap_t cap, word_t length, extra_caps_t excaps, word_t *buffer);
exception_t decodeSetIPCBuffer(cap_t cap, word_t length,
                               cte_t* slot, extra_caps_t excaps, word_t *buffer);
exception_t decodeSetSpace(cap_t cap, word_t length,
                           cte_t* slot, extra_caps_t excaps, word_t *buffer);
exception_t decodeDomainInvocation(word_t invLabel, word_t length,
                                   extra_caps_t excaps, word_t *buffer);
exception_t decodeBindNotification(cap_t cap, extra_caps_t excaps);
exception_t decodeUnbindNotification(cap_t cap);

enum thread_control_flag {
    thread_control_update_priority = 0x1,
    thread_control_update_ipc_buffer = 0x2,
    thread_control_update_space = 0x4,
    thread_control_update_mcp = 0x8,
};

typedef word_t thread_control_flag_t;

exception_t invokeTCB_Suspend(tcb_t *thread);
exception_t invokeTCB_Resume(tcb_t *thread);
exception_t invokeTCB_ThreadControl(tcb_t *target, cte_t* slot, cptr_t faultep,
                                    prio_t mcp, prio_t priority, cap_t cRoot_newCap,
                                    cte_t *cRoot_srcSlot, cap_t vRoot_newCap,
                                    cte_t *vRoot_srcSlot, word_t bufferAddr,
                                    cap_t bufferCap, cte_t *bufferSrcSlot,
                                    thread_control_flag_t updateFlags);
exception_t invokeTCB_CopyRegisters(tcb_t *dest, tcb_t *src,
                                    bool_t suspendSource, bool_t resumeTarget,
                                    bool_t transferFrame, bool_t transferInteger,
                                    word_t transferArch);
exception_t invokeTCB_ReadRegisters(tcb_t *src, bool_t suspendSource,
                                    word_t n, word_t arch, bool_t call);
exception_t invokeTCB_WriteRegisters(tcb_t *dest, bool_t resumeTarget,
                                     word_t n, word_t arch, word_t *buffer);
exception_t invokeTCB_NotificationControl(tcb_t *tcb, notification_t *ntfnPtr);

cptr_t PURE getExtraCPtr(word_t *bufferPtr, word_t i);
void setExtraBadge(word_t *bufferPtr, word_t badge, word_t i);

exception_t lookupExtraCaps(tcb_t* thread, word_t *bufferPtr, seL4_MessageInfo_t info);
word_t setMRs_syscall_error(tcb_t *thread, word_t *receiveIPCBuffer);
word_t CONST Arch_decodeTransfer(word_t flags);
exception_t CONST Arch_performTransfer(word_t arch, tcb_t *tcb_src,
                                       tcb_t *tcb_dest);

#ifdef ENABLE_SMP_SUPPORT
void Arch_migrateTCB(tcb_t *thread);
#endif /* ENABLE_SMP_SUPPORT */

#ifdef CONFIG_DEBUG_BUILD
void setThreadName(tcb_t *thread, const char *name);
#endif /* CONFIG_DEBUG_BUILD */

void Arch_setTCBIPCBuffer(tcb_t *thread, word_t bufferAddr);

#endif
