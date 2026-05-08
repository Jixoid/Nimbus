/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "basis.hh"
#include <expected>
#include <iostream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_map>
#include <istream>
#include <vector>
#include <map>
#include "ds/ds.hh"
#include "qcl/view.hh"
#include "qcl/types.hh"
#include "qcl/object.hh"
#include "qcl/platform.hh"
#include "qcl/dynamicLoad.hh"

#define el else
#define ef else if

using namespace std;



namespace qcl::dyn
{
  map<string, objs> ObjList;
  map<string, effs> EffList;

  void reg(string_view Group, const objs Objs) { ObjList.insert_or_assign(string(Group), Objs); }
  void reg(string_view Group, const effs Effs) { EffList.insert_or_assign(string(Group), Effs); }



  [[gnu::noreturn]] fun __qcl_error(string Msg) -> void { platform::qcl_error(Msg.c_str()); }
  fun __qcl_warning(string Msg) -> void {
    cerr << "qclerr 'dynamicload' {msg: '"+Msg+"'}" << endl;
  }
  fun __qcl_warning(string Msg, string From) -> void {
    cerr << "qclerr 'dynamicload' {msg: '"+Msg+"', from: '"+From+"'}" << endl;
  }
  fun __qcl_warning(string Msg, string From, string MsgEx) -> void {
    cerr << "qclerr 'dynamicload' {msg: '"+Msg+"', from: '"+From+"', info: '"+MsgEx+"'}" << endl;
  }



  fun load_tup(ds::value& Tup /* tuple (type, *name, struct) */, view *Par, string Scope, u32 Index, object* Self = nil) -> object*;
  
  fun load_stc(ds::value& Stc, object *Self, view *Par, string Scope, u32 Index) -> void
  {
    Self->setQDL(Stc);

    auto SelfW = dynamic_cast<view*>(Self);
    
    
    for (u0 i{}; i < Stc.size(); i++) {
      auto paired = Stc.get_stc(i);

      if (SelfW && paired.first == "node") {
        if (!paired.second.isTuple())
        {
          __qcl_warning("invalid property", Scope+"/"+Self->name()+"/"+paired.first, "wrong type, tuple was expected");
          continue;
        }

        load_tup(paired.second, SelfW, Scope+"/"+Self->name(), i);
        continue;
      }

      ef (paired.first == "style") {
        if (!paired.second.isString())
        {
          __qcl_warning("invalid property", Scope+"/"+Self->name()+"/"+paired.first, "wrong type, value was expected");
          continue;
        }
        
        Self->styles().push_back(paired.second.w_string());
        continue;
      }

      ef (paired.first == "styles") {
        if (!paired.second.isArray())
        {
          __qcl_warning("invalid property", Scope+"/"+Self->name()+"/"+paired.first, "wrong type, array was expected");
          continue;
        }

        for (u32 i = 0; i < paired.second.size(); i++)
        {
          auto Obj = paired.second[i];
          if (!Obj.isString())
            throw runtime_error("Wrong type, value was expected: " +Scope+"/"+string(Self->name())+"/styles["+to_string(i)+"]");

          Self->styles().push_back(Obj.w_string());
        }
        continue;
      }
    }
  }

  fun load_tup(ds::value& Tup /* tuple (type, *name, struct) */, view *Par, string Scope, u32 Index, object* Self) -> object*
  {
    // name
    string Name = "__nil__";

    if (auto C = Tup[ds::etype::etStr]; !C.isUnDef())
      Name = C.w_string();


    // type
    pair<string,string> Type;

    if (auto C = Tup[ds::etype::etTyp]; !C.isUnDef())
      Type = C.w_type();
    el
      throw runtime_error("'type' not found: "+Scope+"/"+Name+"/");


    // create || use
    const auto itG = ObjList.find(Type.first);
    if (itG == ObjList.end())
      throw runtime_error("species type not found: " +Scope+"/"+Name+"/&type = \""+Type.first+"::"+Type.second+"\"");

    const auto itO = itG->second.find(Type.second);
    if (itO == itG->second.end())
      throw runtime_error("species type not found: " +Scope+"/"+Name+"/&type = \""+Type.first+"::"+Type.second+"\"");


    if (!Self || !itO->second.second(Self)) {
      if (Self) delete Self;
      Self = itO->second.first();
    }
  
    Self->setParent(Par);
    Self->setName(Name);

    // Absolute
    if (auto C = Tup[ds::etype::etStc]; !C.isUnDef())
      load_stc(C, Self, Par, Scope, Index);

    return Self;
  }


  fun load_prop(object* Ctrl, string Scope, const ds::value& Styles, unordered_map<string, qev_seed> &FuncMap) -> void
  {
    auto FuncGet = [FuncMap](string_view Name) -> expected<qev_seed,bool> {
      auto Fun = FuncMap.find(string(Name));

      return (Fun == FuncMap.end()) ? unexpected(false) : expected<qev_seed,bool>(Fun->second);
    };

    
    // Set Styles
    if (!Styles.isUnDef())
      for (u32 s = 0; s < Ctrl->styles().size(); s++)
      {
        auto Style = Styles[Ctrl->styles()[s]];
        if (Style.isUnDef())
        {
          __qcl_warning("unknown style", Scope+"/"+Ctrl->name(), "The style: "+Ctrl->styles()[s]);
          continue;
        }
        ef (!Style.isStruct())
        {
          __qcl_warning("invalid style", "/root/styles/"+Ctrl->styles()[s], "wrong type, struct was expected");
          continue;
        }


        for (u32 i = 0; i < Style.size(); i++)
        {
          auto prop = Style.get_stc(i);

          auto Err = Ctrl->loadProp(prop.first, prop.second, FuncGet);
          if (Err && *Err)
            continue;

          ef (!Err)
            __qcl_warning("invalid property", Scope+"/"+Ctrl->name()+"/"+prop.first,  Err.error());
            
          ef (Err)
            __qcl_warning("unknown property", Scope+"/"+Ctrl->name()+"/"+prop.first);
        }
      }


    // Set Prop
    if (!Ctrl->qdl().isUnDef())
      for (u32 i = 0; i < Ctrl->qdl().size(); i++)
      {
        auto prop = Ctrl->qdl().get_stc(i);

        if (prop.first == "node" || prop.first == "style" || prop.first == "styles")
          continue;


        auto Err = Ctrl->loadProp(prop.first, prop.second, FuncGet);
        if (Err && *Err)
          continue;

        ef (!Err)
          __qcl_warning("invalid property", Scope+"/"+Ctrl->name()+"/"+prop.first,  Err.error());

        ef (Err)
          __qcl_warning("unknown property", Scope+"/"+Ctrl->name()+"/"+prop.first);
      }

    
    if (auto View = dynamic_cast<view*>(Ctrl))
      for (auto &X: View->childs())
        load_prop(X, Scope+"/"+Ctrl->name(), Styles, FuncMap);
  }



  fun loadFromData(object *Ctrl, sptr<istream> Stream, unordered_map<string, qev_seed> FuncMap) -> object*
  {
    auto DL = ds::value::makeFromStream(Stream.get());


    // import
    if (auto Import = DL["import"]; !Import.isUnDef())
    {
      if (!Import.isArray())
        throw runtime_error("array was expected: /import");
    }


    // root
    if (auto Root = DL["root"]; !Root.isUnDef())
    {
      if (!Root.isTuple())
        throw runtime_error("tuple was expected: /root: ");
      
      Ctrl = load_tup(Root, nil, "", 0, Ctrl);
    }
    

    // loading
    if (auto Styles = DL["styles"]; !Styles.isUnDef())
    {
      if (!Styles.isStruct())
        throw runtime_error("struct was expected: /styles: ");

      load_prop(Ctrl, "", Styles, FuncMap);
    }
    else
      load_prop(Ctrl, "", ds::value::makeUnDef(), FuncMap);

    return Ctrl;
  }

  fun findFromName(view *View, string Name) -> object*
  {
    if (!View)
      return nil;

    for (auto &X: View->childs())
      if (X->name() == Name)
        return X;

      ef (auto C = dynamic_cast<view*>(X); C != nil)
      {
        if (auto B = findFromName(C, Name); B != nil)
          return B;
      }

    return nil;
  }

}
