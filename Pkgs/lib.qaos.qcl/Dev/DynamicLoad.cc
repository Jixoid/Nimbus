/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include "qcl/Types.hh"
#define el else
#define ef else if

#include <memory>
#include <vector>
#include <unordered_map>
#include <map>

#include "Basis.h"

#include "JConf.h"
#include "JConf.hh"

#include "qcl/Control.hh"
#include "qcl/View.hh"
#include "qcl/Effect.hh"
#include "qcl/Popup.hh"
#include "qcl/Platform.hh"
#include "qcl/DynamicLoad.hh"

using namespace std;
using namespace jix;



namespace qcl::dyn
{
  map<string, control* (*)()> RegList;
  map<string, effect*  (*)()> EffList;

  void Register(const char* Name, control* (*Maker)())
  {
    RegList[Name] = Maker;
  }

  void Register(const char* Name, effect* (*Maker)())
  {
    EffList[Name] = Maker;
  }



  [[gnu::noreturn]] void __qcl_error(string Msg)
  {
    platform::qcl_error(Msg.c_str());
  }


  
  void LoadQDL_Arr(const jconf::Value& Arr, view *Par, string Scope);

  void LoadQDL_Stc2(const jconf::Value& Stc, control *Self, view *Par, string Scope, u32 Index)
  {
    Self->QDL = jc_nref(Stc.getHandle());


    jconf::Value Cac;


    // Style
    Cac = Stc["&style"];
    if (Cac.isNull())
    {
      // if (not found) then continue
    }

    ef (Cac.isString())
    {
      string Style = (string)Cac;
      Self->Style.push_back(Style);
    }
    
    ef (Cac.isArray())
    {
      for (u32 i = 0; i < Cac.size(); i++)
      {
        jconf::Value Obj = Cac[i];
        if (!Obj.isString())
          throw runtime_error("Wrong type, value was expected: " +Scope+"/"+Self->Name+"/&style["+to_string(i)+"]");


        string Style = (string)Obj;
        Self->Style.push_back(Style);
      }
    }

    el
      throw runtime_error("Wrong type, value or array was expected: " +Scope+"/"+Self->Name+"/&style");
    
      


    // Childs
    Cac = Stc["&subs"];
    if (!Cac.isNull())
    {
      if (!Cac.isArray())
        throw runtime_error("Wrong type, array was expected: " +Scope+"/"+Self->Name+"/&subs");
      
      else
      {
        if (auto C = dynamic_cast<qcl::view*>(Self); C)
          LoadQDL_Arr(Cac, C, Scope+"/"+Self->Name);
        el
          throw runtime_error("This component is not of type qcl::view: " +Scope+"/"+Self->Name);
      }
    }

  }

  void LoadQDL_Stc(const jconf::Value& Stc, view *Par, string Scope, u32 Index)
  {
    jconf::Value Cac;


    // Name
    Cac = Stc["&name"];
    if (!Cac.isString())
      throw runtime_error("Wrong type, value was expected: " +Scope+ "["+ std::to_string(Index) +"]/&name");

    string Name = (string)Cac;
       


    // Type
    Cac = Stc["&type"];
    if (!Cac.isString())
      throw runtime_error("Wrong type, value was expected: " +Scope+"/"+Name+"/&type");

    string Type = (string)Cac;
       


    // Create New
    auto it = RegList.find(Type);
    if (it == RegList.end())
      throw runtime_error("Species name not found: " +Scope+"/"+Name+"/&type = \""+Type+"\"");


    qsh<control> Self = qsh<control>(it->second());
    Par->Child_Add(Self);

    Self->Name = Name;


    // Absolute
    LoadQDL_Stc2(Stc, Self.get(), Par, Scope, Index);
  }

  void LoadQDL_Arr(const jconf::Value& Arr, view *Par, string Scope)
  {
    vector<shared_ptr<control>> Ret;

    for (u32 i = 0; i < Arr.size(); i++)
    {
      jconf::Value Obj = Arr[i];
      if (!Obj.isStruct())
        throw runtime_error("Wrong type, struct was expected: " +Scope +"["+ std::to_string(i) +"]");

      
      LoadQDL_Stc(Obj, Par, Scope, i);
    }
  }



  pair<bool, shared_ptr<popup_item>> __popup_load_items(const jconf::Value& Node, string Scope, unordered_map<string, qev_seed> &FuncMap)
  {
    if (!Node.isArray())
      return {false, {}};


    shared_ptr<popup_item> Ret = make_shared<popup_item>();


    for (u32 i = 0; i < Node.size(); i++)
    {
      jconf::Value Sub = Node[i];

      if (Sub.isString())
      {
        string Cac = (string)Sub;

        shared_ptr<popup_item> Tmp = make_shared<popup_item>();
        Tmp->Name = Cac;

        Ret->Items.push_back(Tmp);
      }

      ef (Sub.isStruct())
      {
        auto VNa = Sub["&name"];
        if (VNa.isNull())
          return {true, {}};

        string Cac = (string)VNa;
        shared_ptr<popup_item> Tmp = make_shared<popup_item>();
        Tmp->Name = Cac;


        auto VIt = Sub["&subs"];
        if (!VIt.isNull())
        {
          auto SThe = __popup_load_items(VIt, Scope+"/"+Cac, FuncMap);

          if (!SThe.first)
            return {false, {}};

          Tmp = SThe.second;
          Tmp->Name = Cac;

          
          for (auto &X: Tmp->Items)
            X->Parent = Ret.get();
        }



        auto VOnCl = Sub["^OnClick"];
        if (!VOnCl.isNull())
        {
          string Prop_Func = (string)VOnCl;


          auto it = FuncMap.find(Prop_Func);

          if (it == FuncMap.end())
            throw runtime_error("Func not found in code: " +Scope+"/"+"OnClick" +" = "+ Prop_Func);


          it->second.ToLoad(Tmp->OnClick);
        }


        Ret->Items.push_back(std::move(Tmp));
      }

      else
        return {false, {}};
    }

    return {true, Ret};
  }


  void LoadQDL_Popup(popup *Ctrl, string Scope, unordered_map<string, qev_seed> &FuncMap)
  {
    jconf::Value Node = jconf::Value(Ctrl->QDL, false)["Items"];
    // Assuming Ctrl->QDL is jc_obj/stc handle, we wrap it.
    
    if (Node.isNull())
      return;

    ef (!Node.isArray())
      throw runtime_error("Invalid prop: " +Scope+"/"+Ctrl->Name+"/"+"Items");
    


    auto Temp = __popup_load_items(Node, Scope+"/"+Ctrl->Name+"/Items", FuncMap);
    if (!Temp.first)
      throw runtime_error("Invalid prop: " +Scope+"/"+Ctrl->Name+"/"+"Items");


    Ctrl->Items = Temp.second;
    Ctrl->VItems = Ctrl->Items.get();
  }

  void LoadQDL_Prop(control* Ctrl, string Scope, const jconf::Value& Styles, unordered_map<string, qev_seed> &FuncMap)
  {
    // Set Styles
    for (u32 s = 0; s < Ctrl->Style.size(); s++)
    {
      jconf::Value Style = Styles[Ctrl->Style[s]];
      if (Style.isNull())
        throw runtime_error("struct was expected: root/&styles/"+Ctrl->Style[s]);

      else if (!Style.isStruct())
        throw runtime_error("Wrong type, struct was expected: root/&styles/"+Ctrl->Style[s]);


      jc_obj H = Style.getHandle();
      for (u32 i = 0; i < jc_stc_count(H); i++)
      {
        char *CStr;
        string Prop_Name;

        jconf::Value Prop(jc_stc_index(H, i, &CStr));
        Prop_Name = string(CStr);
        jc_dis_str(CStr);


        if (Prop_Name[0] == '&')
          continue;

        
        if (Prop_Name[0] == '^')
        {
          string Prop_Func;

          Prop_Func = (string)Prop;


          auto it = FuncMap.find(Prop_Func);

          if (it == FuncMap.end())
            throw runtime_error("Func not found in code: " +Scope+"/"+Ctrl->Name+"("+Ctrl->Style[s]+")"+"/"+Prop_Name +" = "+ Prop_Func);


          if (!Ctrl->LoadFunc(Prop_Name.substr(1), it->second))
            throw runtime_error("Invalid prop: " +Scope+"/"+Ctrl->Name+"("+Ctrl->Style[s]+")"+"/"+Prop_Name);
        }

        ef (auto Err = Ctrl->LoadProp(Prop_Name, Prop); Err.Type != propError::peOK) switch (Err.Type)
          {
            case propError::peOK: break;
            case propError::peUnknown: __qcl_error("Unknown prop: " +Scope+"/"+Ctrl->Name+"("+Ctrl->Style[s]+")"+"/"+Prop_Name); break;
            case propError::peInvalid: __qcl_error("Invalid prop: " +Scope+"/"+Ctrl->Name+"("+Ctrl->Style[s]+")"+"/"+Prop_Name +", Error: "+Err.Msg); break;
          }
      }

    }


    // Set Prop
    if (Ctrl->QDL != Nil) 
    {
      jc_obj H = Ctrl->QDL;
      for (u32 i = 0; i < jc_stc_count(H); i++)
      {
        char *CStr;
        string Prop_Name;

        jconf::Value Prop(jc_stc_index(H, i, &CStr));
        Prop_Name = string(CStr);
        jc_dis_str(CStr);
        

        if (Prop_Name[0] == '&')
          continue;


      
        if (Prop_Name[0] == '^')
        {
          string Prop_Func;

          Prop_Func = (string)Prop;


          auto it = FuncMap.find(Prop_Func);

          if (it == FuncMap.end())
            throw runtime_error("Func not found in code: " +Scope+"/"+Ctrl->Name+"/"+Prop_Name +" = "+ Prop_Func);


          if (!Ctrl->LoadFunc(Prop_Name.substr(1), it->second))
            throw runtime_error("Invalid prop: " +Scope+"/"+Ctrl->Name+"/"+Prop_Name);
        }

        ef (auto C = dynamic_cast<qcl::popup*>(Ctrl); C != Nil && Prop_Name == "Items")
          {}

        ef (auto Err = Ctrl->LoadProp(Prop_Name, Prop); Err.Type != propError::peOK) switch (Err.Type)
          {
            case propError::peOK: break;
            case propError::peUnknown: __qcl_error("Unknown prop: " +Scope+"/"+Ctrl->Name+"/"+Prop_Name); break;
            case propError::peInvalid: __qcl_error("Invalid prop: " +Scope+"/"+Ctrl->Name+"/"+Prop_Name +", Error: "+Err.Msg); break;
          }
      }


      // Popup
      if (auto C = dynamic_cast<qcl::popup*>(Ctrl); C != Nil)
        LoadQDL_Popup(C, Scope, FuncMap);


      if (auto C = dynamic_cast<qcl::view*>(Ctrl); C != Nil)
        for (auto &X: C->Childs)
          LoadQDL_Prop(X.get(), Scope+"/"+Ctrl->Name, Styles, FuncMap);
    }
  }


  void loadFromFile(control *Ctrl, string FPath, unordered_map<string, qev_seed> FuncMap)
  {
    // Open File
    jconf::Value DL = jconf::Value::Parse(FPath);
    if (DL.isNull())
      throw runtime_error("Corrupt qdl file: " +FPath);



    // Root
    jconf::Value Root = DL["&root"];
    if (!Root.isStruct())
      throw runtime_error("struct was expected: /&root: " +FPath);

    LoadQDL_Stc2(Root, Ctrl, Nil, "", 0);

    
    // Load Design
    jconf::Value Styles = DL["&styles"];
    if (!Styles.isStruct())
      throw runtime_error("struct was expected: /&styles: " +FPath);
    
    LoadQDL_Prop(Ctrl, "", Styles, FuncMap);
  }



  qsh<control> findFromName(view *View, string Name)
  {
    for (auto &X: View->Childs)
      if (X->Name == Name)
        return X;

      ef (auto C = dynamic_cast<view*>(X.get()); C != Nil)
      {
        if (auto B = findFromName(C, Name); B != Nil)
          return B;
      }

    return Nil;
  }

}
