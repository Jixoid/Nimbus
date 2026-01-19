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

#include "Basis.h"

#include "JConf.h"
#include "JConf.hh"

#include "qcl/Control.hh"
#include "qcl/Graphic.hh"
#include "qcl/Window.hh"
#include "qcl/Effect.hh"
#include "qcl/Application.hh"
#include "qcl/DynamicLoad.hh"
#include "qcl/Popup.hh"
#include "qcl/View.hh"

using namespace std;
using namespace jix;



u64 GetTickCount()
{
  struct timeval tv;
  gettimeofday(&tv, nullptr);
  
  return static_cast<u64>(tv.tv_sec) *1000 +tv.tv_usec /1000;
}


namespace qcl
{

  control::control()
  {
    Surface = qsh<qcl::surface>(new qcl::surface(Size.W, Size.H));
  }

  control::~control()
  {}



  void control::Draw_before()
  {
    Surface->Clear();
  }

  void control::Draw_after()
  {}

  void control::Draw()
  {}


  void control::CalcAutoSize()
  { PreferedSize = {20,20}; }


  poit_i32 GetLocalPos(view *Target, control *This)
  {
    poit_i32 Ret{0,0}; 

    while (true)
    {
      Ret += This->Poit;

      if (This->Parent == Nil)
        return {0,0};

      ef (This->Parent == Target)
        return Ret;

      el
        This = This->Parent;
    }
    
  }

  bool control::Show_Popup(poit_f32 Poit)
  {
    if (Popup == Nil)
      return false;
      

    window *Window = Nil;
    {
      control *Parent = GetRoot();

      if (Parent == Nil)
        return false;

      ef (dynamic_cast<window*>(Parent) == Nil)
        return false;

      Window = static_cast<window*>(Parent);
    }


    Popup->Poit = GetLocalPos(Window, this) +poit_i32{(i32)Poit.X, (i32)Poit.Y};
    Popup->Flag_Add(DirtyRebound);
    Window->Overlay = Popup;
  

    CurrentApp->PushMessage(Window, controlMessages::cmPaint);

    return true;
  }


  control* control::GetRoot()
  {
    control* This = this;

    while (This->Parent != Nil)
      This = This->Parent;

    return This;
  }

  void control::DyeToRoot(dirtyFlags Flag)
  {
    control* This = this;
    This->Flag_Add(Flag);

    while (This->Parent != Nil)
    {
      This = This->Parent;

      This->Flag_Add(Flag);
    }
  }


  void control::Flag_Add(dirtyFlagSet Flag)
  {
    if (Flag & DirtyResize)
      Flag |= (DirtyRebound | DirtyTiling);

      
    if (Flag & DirtyTiling)
      Flag |= (DirtyDraw);
      
      
    if (Flag & (DirtyDraw))
    {
      control* This = this;
      
      while (This->Parent != Nil)
      {
        This = This->Parent;
        This->DirtyFlags |= (DirtyDraw);
      }
    }

    if (Flag & (DirtyResize | DirtyRebound))
    {
      control* This = this;
      
      while (This->Parent != Nil)
      {
        This = This->Parent;
        
        if (This->AutoSize)
          This->DirtyFlags |= (DirtyAutoSize);
        
        This->DirtyFlags |= (DirtyTiling | DirtyDraw);
      }
    }

    DirtyFlags |= Flag;
  }

  void control::Flag_Rem(dirtyFlagSet Flag)
  {
    DirtyFlags &= ~Flag;
  }



  control* __findInScope(control* Self, string Name)
  {
    if (Self->Parent == Nil)
      return Nil;


    if (Self->Parent->Name == Name)
      return Self->Parent;

    
    for (auto &X: Self->Parent->Childs)
      if (X.get() != Self && X->Name == Name)
        return X.get();

    return Nil;
  }


  propError control::LoadProp(string Name, const jconf::Value& Prop)
  {

    if (Name == "Effects")
    {
      vector<qsh<effect>> Nat;

      if (!Prop.isArray())
        return propError::peInvalid;
      

      for (u32 i = 0; i < Prop.size(); i++)
      {
        jconf::Value X = Prop[i];

        if (!X.isStruct())
          return propError::peInvalid;


        
        auto Cac = (string)X["&type"];
        if (Cac.empty())
          return propError::peInvalid;

        
        auto it = dyn::EffList.find(Cac);
        if (it == dyn::EffList.end())
          return propError::peInvalid;

        Nat.push_back(qsh<effect>(it->second()));


        
        jc_obj H = X.getHandle();
        for (u32 j = 0; j < jc_stc_count(H); j++)
        {
          char *Te;
          auto SX = jconf::Value(jc_stc_index(H, j, &Te));
          
          Cac = string(Te);
          jc_dis_str(Te);

          if (Cac == "&type")
            continue;

          if (auto Err = Nat.back()->LoadProp(Cac, SX); Err.Type != propError::peOK)
            return Err;
        }

      }
    

      Effects = (Nat);
      return propError::peOK;
    }

    ef (Name == "Poit")
    {
      if (!Prop.isStruct())
        return propError::peInvalid;

      if (auto Buf = Prop["x"]; !Buf.isNull())
        Poit.X = (i64)Buf;

      if (auto Buf = Prop["y"]; !Buf.isNull())
        Poit.Y = (i64)Buf;

      Flag_Add(DirtyRebound);
      return propError::peOK;
    }

    ef (Name == "Size")
    {
      if (!Prop.isStruct())
        return propError::peInvalid;

      if (auto Buf = Prop["w"]; !Buf.isNull())
        Size.W = (i64)Buf;

      if (auto Buf = Prop["h"]; !Buf.isNull())
        Size.H = (i64)Buf;

      Flag_Add(DirtyResize | DirtyRebound);
      return propError::peOK;
    }

    ef (Name == "MinSize")
    {
      if (!Prop.isStruct())
        return propError::peInvalid;

      if (auto Buf = Prop["w"]; !Buf.isNull())
        MinSize.W = (i64)Buf;

      if (auto Buf = Prop["h"]; !Buf.isNull())
        MinSize.H = (i64)Buf;

      //Flag_Add(DirtyResize | DirtyRebound);
      return propError::peOK;
    }

    ef (Name == "MaxSize")
    {
      if (!Prop.isStruct())
        return propError::peInvalid;

      if (auto Buf = Prop["w"]; !Buf.isNull())
        MaxSize.W = (i64)Buf;

      if (auto Buf = Prop["h"]; !Buf.isNull())
        MaxSize.H = (i64)Buf;

      //Flag_Add(DirtyResize | DirtyRebound);
      return propError::peOK;
    }

    ef (Name == "Tag")
    {
      if (!Prop.isInt())
        return propError::peInvalid;

      Tag = (i64)Prop;
      
      return propError::peOK;
    }

    ef (Name == "Visible")
    {
      if (!Prop.isBool())
        return propError::peInvalid;

      Visible = (bool)Prop;

      if (Parent != Nil)
        Parent->Flag_Add(DirtyTiling);

      return propError::peOK;
    }

    ef (Name == "Enabled")
    {
      if (!Prop.isBool())
        return propError::peInvalid;

      Enabled = (bool)Prop;

      Flag_Add(DirtyDraw);
      return propError::peOK;
    }

    ef (Name == "AutoSize")
    {
      if (!Prop.isBool())
        return propError::peInvalid;

      AutoSize = (bool)Prop;

      if (Parent != Nil)
        Parent->Flag_Add(DirtyTiling);

      return propError::peOK;
    }

    ef (Name == "Transparent")
    {
      if (!Prop.isBool())
        return propError::peInvalid;

      Transparent = (bool)Prop;

      return propError::peOK;
    }

    ef (Name == "Opacity")
    {
      if (!Prop.isFloat())
        return propError::peInvalid;

      Opacity = (f64)Prop;

      return propError::peOK;
    }

    ef (Name == "Anchors")
    {
      controlAnchors Nat = this->Anchors;

      if (!Prop.isStruct())
        return propError::peInvalid;


      auto Handler = [this](control* This, const jconf::Value& Stc, controlAnchor* Anchor) -> pair<bool, string>
      {
        if (!Stc.isStruct())
          return {false," is not a struct"};

        
        // Active
        if (auto Buf = Stc["active"]; !Buf.isNull())
          Anchor->Active = (bool)Stc["active"];

        // Control
        if (auto Buf = Stc["control"]; !Buf.isNull())
        {
          string Cac = (string)Buf;

          if (Cac[0] != '@')
            return {false, "/control is not a citation"};

          if (Cac == "@^")
            Anchor->Control = this->Parent;
          el
            Anchor->Control = __findInScope(This, Cac.substr(1));

          if (Anchor->Control == Nil)
            return {false, "/control is not finded"};
        }

        // Size
        if (auto Buf = Stc["side"]; !Buf.isNull())
        {
          string Cac = (string)Buf;

          if (Cac == "beg" || Cac == "<-")
            Anchor->Side = controlAnchorSide::casBegin;

          ef (Cac == "end" || Cac == "->")
            Anchor->Side = controlAnchorSide::casEnd;

          else
            return {false, "/side (use 'beg' | 'end')"};
        }

        return {true, ""};
      };


      if (auto Buf = Prop["left"]; !Buf.isNull())
        if (auto Err = Handler(this, Buf, &Nat.Left); !Err.first)
          return propError(propError::peInvalid, "Invalid: /left" +Err.second);

      if (auto Buf = Prop["top"]; !Buf.isNull())
        if (auto Err = Handler(this, Buf, &Nat.Top); !Err.first)
          return propError(propError::peInvalid, "Invalid: /top" +Err.second);

      if (auto Buf = Prop["righ"]; !Buf.isNull())
        if (auto Err = Handler(this, Buf, &Nat.Righ); !Err.first)
          return propError(propError::peInvalid, "Invalid: /righ" +Err.second);

      if (auto Buf = Prop["bot"]; !Buf.isNull())
        if (auto Err = Handler(this, Buf, &Nat.Bot); !Err.first)
          return propError(propError::peInvalid, "Invalid: /bot" +Err.second);


      Anchors = (Nat);

      if (Parent != Nil)
        Parent->Flag_Add(DirtyTiling);

      return propError::peOK;
    }

    ef (Name == "Margins")
    {
      if (!Prop.isStruct())
        return propError::peInvalid;


      if (auto Buf = Prop["left"]; !Buf.isNull())
        Margins.X1 = (i64)Buf;

      if (auto Buf = Prop["righ"]; !Buf.isNull())
        Margins.X2 = (i64)Buf;

      if (auto Buf = Prop["top"]; !Buf.isNull())
        Margins.Y1 = (i64)Buf;

      if (auto Buf = Prop["bot"]; !Buf.isNull())
        Margins.Y2 = (i64)Buf;

      if (Parent != Nil)
        Parent->Flag_Add(DirtyTiling);

      return propError::peOK;
    }

    ef (Name == "Paddings")
    {
      if (!Prop.isStruct())
        return propError::peInvalid;


      if (auto Buf = Prop["left"]; !Buf.isNull())
        Paddings.X1 = (i64)Buf;

      if (auto Buf = Prop["righ"]; !Buf.isNull())
        Paddings.X2 = (i64)Buf;

      if (auto Buf = Prop["top"]; !Buf.isNull())
        Paddings.Y1 = (i64)Buf;

      if (auto Buf = Prop["bot"]; !Buf.isNull())
        Paddings.Y2 = (i64)Buf;

      Flag_Add(DirtyTiling);
      return propError::peOK;
    }

    else
      return propError::peUnknown;
  }

  bool control::LoadFunc(string Name, qev_seed FuncSeed)
  {
    #define makro(X) (Name == #X) { FuncSeed.ToLoad(X); return true; }


    if makro(OnPaint)
    ef makro(OnPaint_before)
    ef makro(OnPaint_after)
    ef makro(OnResize)
    ef makro(OnClick)
    ef makro(OnClickEx)
    ef makro(OnDblClick)
    ef makro(OnMouseDown)
    ef makro(OnMouseUp)
    ef makro(OnMouseMove)
    ef makro(OnKeyDown)
    ef makro(OnKeyUp)
    ef makro(OnScrollVert)
    ef makro(OnScrollHorz)
    ef makro(OnStateChanged)

    #undef makro

    return false;
  }


  void control::Handler_Message(controlMessages Msg)
  {
    switch (Msg)
    {
      case controlMessages::cmPaint: Handler_Paint(); break;
      case controlMessages::cmReset: Do_Reset(); break;

      default: break;
    }
  }

  void control::Handler_Paint()
  {
    Do_Paint_prepare();
    Do_Paint();
  }

  void control::Handler_Resize()
  {
    Flag_Add(DirtyResize);
  }

  void control::Handler_MouseDown(poit_i32 Pos, shiftStateSet Button, shiftStateSet State)
  {
    Do_MouseDown(Pos, Button, State);
  }
  
  void control::Handler_MouseUp(poit_i32 Pos, shiftStateSet Button, shiftStateSet State)
  {
    Do_MouseUp(Pos, Button, State);
  }

  void control::Handler_MouseMove(poit_i32 Pos, shiftStateSet State)
  {
    Do_MouseMove(Pos, State);
  }

  void control::Handler_KeyDown(char *Key, u32 KeyCode, shiftStateSet State)
  {
    Do_KeyDown(Key, KeyCode, State);
  }
  
  void control::Handler_KeyUp(char *Key, u32 KeyCode, shiftStateSet State)
  {
    Do_KeyUp(Key, KeyCode, State);
  }

  void control::Handler_ScrollVert(poit_i32 Pos, i16 Delta, shiftStateSet State)
  {
    Do_ScrollVert(Pos, Delta, State);
  }

  void control::Handler_ScrollHorz(poit_i32 Pos, i16 Delta, shiftStateSet State)
  {
    Do_ScrollHorz(Pos, Delta, State);
  }

  void control::Handler_StateChanged(controlStateSet State)
  {
    ControlState = State;

    Do_StateChanged(State);
  }



  void control::Do_Reset()
  {
    Handler_Paint();
  }

  void control::Do_Paint()
  {
    Draw_before();
    OnPaint_before.Call(this);

    Draw();
    OnPaint.Call(this);

    Draw_after();
    OnPaint_after.Call(this);


    if (!Enabled)
      Surface->Filter_GrayScale({0,0, (f32)Size.W, (f32)Size.H});

    Surface->Flush();

    Flag_Rem(DirtyDraw);
  }

  void control::Do_Paint_prepare()
  {
    if (Flag_HasR(DirtyResize))
      Do_Resize();

    if (Flag_HasR(DirtyRebound))
      EndPoit = {
        .X = Poit.X +Size.W,
        .Y = Poit.Y +Size.H,
      };

  }

  void control::Do_Resize()
  {
    Surface->Set_Size(max<i32>(0,Size.W), max<i32>(0,Size.H));

    OnResize.Call(this);
  }

  void control::Do_Click()
  {
    OnClick.Call(this);
  }

  void control::Do_ClickEx(poit_i32 Pos)
  {
    OnClickEx.Call(this, Pos);
  }

  void control::Do_DblClick()
  {
    OnDblClick.Call(this);
  }

  void control::Do_MouseDown(poit_i32 Pos, shiftStateSet Button, shiftStateSet State)
  {
    OnMouseDown.Call(this, Pos, Button, State);
  }

  void control::Do_MouseUp(poit_i32 Pos, shiftStateSet Button, shiftStateSet State)
  {
    OnMouseUp.Call(this, Pos, Button, State);


    // Click
    if (State != 0 || Button != shiftStates::ssLeft)
      return;

    if (
      Pos.X < 0 ||
      Pos.Y < 0 ||
      Pos.X > Size.W ||
      Pos.Y > Size.H
    )
      return;


    if (OnDblClick || GetTickCount() -LastClick > 300)
    {
      Do_ClickEx(Pos);
      Do_Click();

      LastClick = GetTickCount();
    }
    else
    {
      Do_DblClick();
    }

  }

  void control::Do_MouseMove (poit_i32 Pos, shiftStateSet State)
  {
    OnMouseMove.Call(this, Pos, State);
  }

  void control::Do_KeyDown(char *Key, u32 KeyCode, shiftStateSet State)
  {
    OnKeyDown.Call(this, Key, KeyCode, State);
  }

  void control::Do_KeyUp(char *Key, u32 KeyCode, shiftStateSet State)
  {
    OnKeyUp.Call(this, Key, KeyCode, State);
  }

  void control::Do_ScrollVert(poit_i32 Pos, i16 Delta, shiftStateSet State)
  {
    OnScrollVert.Call(this, Pos, Delta, State);
  }

  void control::Do_ScrollHorz(poit_i32 Pos, i16 Delta, shiftStateSet State)
  {
    OnScrollHorz.Call(this, Pos, Delta, State);
  }

  void control::Do_StateChanged(controlStateSet State)
  {
    OnStateChanged.Call(this, State);
  }

}
