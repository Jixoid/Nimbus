/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file popup.hh
 * @brief Context menu interfaces and structures.
 */

#pragma once

#include "basis.hh"
#include "qcl/types.hh"
#include "qcl/visual.hh"

using namespace std;



namespace qcl
{

  /**
   * @brief Base class for popup context menus.
   */
  class contextMenu: public visual
  {
    public:
      /**
       * @brief Default constructor for context menu.
       */
      inline constexpr contextMenu() {}
      
      /**
       * @brief Virtual destructor for context menu.
       */
      inline constexpr ~contextMenu() override {}

    public:
      /**
       * @brief Displays the context menu at a specific screen coordinate.
       * 
       * @param Val The position to show the popup at.
       */
      virtual fun showPopup(poit<i32> Val) -> void = 0;
  };

}
