/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <map>
#include <unordered_map>

#include "Basis.h"

#include "qcl/Control.hh"
#include "qcl/Types.hh"
#include "qcl/View.hh"
#include "qcl/Effect.hh"


using namespace std;
using namespace jix;



namespace qcl::dyn
{
  extern map<string, control* (*)()> RegList;
  extern map<string, effect*  (*)()> EffList;

  void Register(const char* Name, control* (*Maker)());
  void Register(const char* Name, effect*  (*Maker)());


  
  void loadFromFile(control *Ctrl, string FPath, unordered_map<string, qev_seed> FuncMap);
  void loadFromData(control *Ctrl, data_ Data, unordered_map<string, qev_seed> FuncMap);

  qsh<control> findFromName(view *View, string Name);
}
