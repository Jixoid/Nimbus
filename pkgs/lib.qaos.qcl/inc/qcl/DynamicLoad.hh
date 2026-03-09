/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <istream>
#include <map>
#include <string_view>
#include <unordered_map>

#include "qcl/Object.hh"
#include "qcl/Types.hh"
#include "qcl/View.hh"
#include "qcl/Effect.hh"


using namespace std;



namespace qcl::dyn
{
  using objs = map<string_view, pair<fun(*)() -> object*, fun(*)(object*) -> bool>>;
  using effs = map<string_view, pair<fun(*)() -> effect*, fun(*)(effect*) -> bool>>;
  
  extern map<string, objs> ObjList;
  extern map<string, effs> EffList;

  fun reg(string_view, const objs) -> void;
  fun reg(string_view, const effs) -> void;

  fun loadFromData(object *Ctrl, sptr<istream> Data, unordered_map<string, qev_seed> FuncMap) -> object*;
  fun findFromName(view *View, string Name) -> object*;
}
