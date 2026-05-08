/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file dynamicLoad.hh
 * @brief Dynamic loading system for QCL objects and effects.
 */

#pragma once

#include <istream>
#include <map>
#include <string_view>
#include <unordered_map>
#include "qcl/object.hh"
#include "qcl/types.hh"
#include "qcl/view.hh"
#include "qcl/effect.hh"

using namespace std;



namespace qcl::dyn
{
  /**
   * @brief Type definition for object registration maps.
   */
  using objs = map<string_view, pair<fun(*)() -> object*, fun(*)(object*) -> bool>>;

  /**
   * @brief Type definition for effect registration maps.
   */
  using effs = map<string_view, pair<fun(*)() -> effect*, fun(*)(effect*) -> bool>>;
  
  /**
   * @brief Global list of registered objects.
   */
  extern map<string, objs> ObjList;

  /**
   * @brief Global list of registered effects.
   */
  extern map<string, effs> EffList;

  /**
   * @brief Registers object creators.
   */
  fun reg(string_view, const objs) -> void;

  /**
   * @brief Registers effect creators.
   */
  fun reg(string_view, const effs) -> void;

  /**
   * @brief Loads an object hierarchy from a data stream.
   */
  fun loadFromData(object *Ctrl, sptr<istream> Data, unordered_map<string, qev_seed> FuncMap) -> object*;

  /**
   * @brief Finds an object by its name within a view hierarchy.
   */
  fun findFromName(view *View, string Name) -> object*;
}
