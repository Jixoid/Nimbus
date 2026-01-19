/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"

using namespace jix;



namespace qcl
{

  enum shiftStates: u32
  {
    ssLeft     = (u32)1 << 0,
    ssRight    = (u32)1 << 1,
    ssMiddle   = (u32)1 << 2,
    
    ssCtrl     = (u32)1 << 3,
    ssShift    = (u32)1 << 4,
    ssAlt      = (u32)1 << 5,
    ssSuper    = (u32)1 << 6,
    ssAltGr    = (u32)1 << 7,
  };
  using shiftStateSet = u32;

}
