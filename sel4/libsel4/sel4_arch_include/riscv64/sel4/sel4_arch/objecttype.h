/*
 * Copyright 2018, Data61
 * Commonwealth Scientific and Industrial Research Organisation (CSIRO)
 * ABN 41 687 119 230.
 *
 * This software may be distributed and modified according to the terms of
 * the GNU General Public License version 2. Note that NO WARRANTY is provided.
 * See "LICENSE_GPLv2.txt" for details.
 *
 * @TAG(DATA61_GPL)
 */

/*
 *
 * Copyright 2016, 2017 Hesham Almatary, Data61/CSIRO <hesham.almatary@data61.csiro.au>
 * Copyright 2015, 2016 Hesham Almatary <heshamelmatary@gmail.com>
 */

#ifndef __SEL4_ARCH_OBJECT_TYPE_H
#define __SEL4_ARCH_OBJECT_TYPE_H

#ifdef HAVE_AUTOCONF
#include <autoconf.h>
#endif /* HAVE_AUTOCONF */

typedef enum _mode_object {
    seL4_RISCV_Giga_Page = seL4_NonArchObjectTypeCount,
#if CONFIG_PT_LEVELS > 3
    seL4_RISCV_Tera_Page,
#endif
    seL4_ModeObjectTypeCount
} seL4_ModeObjectType;

#if CONFIG_PT_LEVELS <= 3
#define seL4_RISCV_Tera_Page 0xffffffff
#endif

#endif
