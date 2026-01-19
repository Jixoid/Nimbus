/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include <memory>

#include "qcl/Control.hh"
#include "qcl/Popup.hh"

using namespace std;
using namespace jix;



namespace qcl
{

  popup::popup()
    : control()
    , Items(make_shared<popup_item>())
    , VItems(Items.get())
  {}

  popup::~popup()
  {}
  
}
