/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/basis.hh"
#include "qcl/dynamicLoad.hh"
#include "qstd/standard.hh"

#define el else
#define ef else if

using namespace std;
using namespace qcl;



namespace qstd
{
  #define regO(X) {#X, {[]() -> object* { return new X(); }, [](object* val) -> bool { return dynamic_cast<X*>(val); }}}

  const dyn::objs _objs = {
    {"empty", {[]() -> object* { return new qcl::widget; }, [](object* val) -> bool { return dynamic_cast<qcl::widget*>(val); }}},
    regO(form),
    regO(layout),
    regO(layout_vert),
    regO(layout_horz),
    regO(layout_flow),
    regO(text),
    regO(icon),
    regO(edit),
    regO(radio),
    regO(check),
    regO(toggle),
    regO(progbar),
    regO(slider),
    regO(tabs),
    regO(choice),
    regO(grid),
    regO(popupMenu),
    regO(chip),
    regO(card),
    regO(multi_progbar),
    regO(interval),
  };

  #undef regO

  fun objs() -> dyn::objs { return _objs; }

}
