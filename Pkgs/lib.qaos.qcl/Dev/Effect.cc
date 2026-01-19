/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#define el else
#define ef else if

#include "Basis.h"

#include "JConf.hh"

#include "qcl/Effect.hh"

using namespace std;
using namespace jix;



namespace qcl
{

  effect::effect()
  {}

  effect::~effect()
  {}



  propError effect::LoadProp(string Name, const jconf::Value& Prop)
  {

    if (Name == "Enabled")
    {
      bool Nat = this->Enabled;
      
      if (!Prop.isBool())
        return propError::peInvalid;


      Nat = (bool)Prop;

      Enabled = (Nat);
      return propError::peOK;
    }

    else
      return propError::peUnknown;

  }

}

