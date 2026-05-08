/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qcl/platform.hh"

#ifndef __unix__
#include <stdexcept>
#endif

using namespace std;



namespace qcl::platform
{
  #ifndef __unix__
  extern "C" [[gnu::noreturn]] fun qcl_error(const char *Msg) -> void
  {
    throw runtime_error(Msg);
  }
  #endif

  api *API{};
}
