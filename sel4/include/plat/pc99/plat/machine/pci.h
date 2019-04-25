/*
 * Copyright 2014, General Dynamics C4 Systems
 *
 * This software may be distributed and modified according to the terms of
 * the GNU General Public License version 2. Note that NO WARRANTY is provided.
 * See "LICENSE_GPLv2.txt" for details.
 *
 * @TAG(GD_GPL)
 */

#ifndef __PLAT_MACHINE_PCI_H
#define __PLAT_MACHINE_PCI_H

#define get_pci_bus(x) (((x)>>8u) & 0xffu)
#define get_pci_dev(x) (((x)>>3u) & 0x1fu)
#define get_pci_fun(x) ((x) & 0x7u)
#define get_dev_id(bus, dev, fun) (((bus) << 8u) | ((dev) << 3u) | (fun))

#define PCI_BUS_MAX     255
#define PCI_DEV_MAX     31
#define PCI_FUNC_MAX    7

#endif
