/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <string_view>
#include "ds/ds.hh"
#include "qcl/effect.hh"

#define el else
#define ef else if

using namespace std;



namespace qcl
{

  fun effect::loadProp(string_view Name, const ds::value &Prop) -> std::expected<bool, std::string>
  {
    if (Name == "enabled")
    {
      if (!Prop.isBool()) return std::unexpected("was expected bool");


      setEnabled(Prop.w_bool());
      return true;
    }

    el return std::unexpected("");
  }

}

