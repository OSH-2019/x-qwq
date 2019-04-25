/*
 * Copyright 2014, General Dynamics C4 Systems
 *
 * This software may be distributed and modified according to the terms of
 * the GNU General Public License version 2. Note that NO WARRANTY is provided.
 * See "LICENSE_GPLv2.txt" for details.
 *
 * @TAG(GD_GPL)
 */

#ifndef __ARCH_KERNEL_THREAD_H
#define __ARCH_KERNEL_THREAD_H

#include <object.h>

void Arch_switchToThread(tcb_t *tcb);
void Arch_switchToIdleThread(void);
void Arch_configureIdleThread(tcb_t *tcb);
void Arch_activateIdleThread(tcb_t *tcb);
word_t sanitiseRegister(register_t reg, word_t v, bool_t archInfo);

static inline bool_t CONST
Arch_getSanitiseRegisterInfo(tcb_t *thread)
{
    return 0;
}

void Mode_postModifyRegisters(tcb_t *tptr);

#endif
