/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.hh"
#include "qcl/Types.hh"
#include "qcl/Visual.hh"

using namespace std;



namespace qcl
{

  class contextMenu: public visual
  {
    public:
      inline constexpr contextMenu() {}
      inline constexpr ~contextMenu() override {}

    public:
      virtual fun showPopup(poit<i32> Val) -> void = 0;
  };

}
