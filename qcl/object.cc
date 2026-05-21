/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <cassert>
#include "qcl/types.hh"
#include "qcl/view.hh"
#include "qcl/object.hh"

using namespace std;



namespace qcl
{
  
  fun object::setParent(view* Val) -> void
  {
    if (m_parent)
      assert(m_parent->relChild(this) && "parent-child relationship desynced!");

    if (Val)
      Val->addChild(this);
  }


  fun object::getRoot() noexcept -> view*
  {
    object* This{this};

    while (This->parent())
      This = This->parent();

    return dynamic_cast<view*>(This);
  }
  
}
