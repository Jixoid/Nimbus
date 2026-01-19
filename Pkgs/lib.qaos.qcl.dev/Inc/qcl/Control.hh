/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <vector>

#include <sys/time.h>

#include "Basis.h"

#include "JConf.h"
#include "JConf.hh"

#include "qcl/Graphic.hh"
#include "qcl/Effect.hh"
#include "qcl/Input.hh"
#include "qcl/Types.hh"

using namespace std;
using namespace jix;



namespace qcl
{
  class view;  // Forward
  class popup; // Forward


  enum dirtyFlags: u32
  {
    DirtyTiling   = (u32)1 << 0,
    DirtyAutoSize = (u32)1 << 1,
    DirtyResize   = (u32)1 << 2,
    DirtyRebound  = (u32)1 << 3,
    DirtyDraw     = (u32)1 << 4,
  };
  using dirtyFlagSet = u32;


  enum controlStates: u16
  {
    csHover  = (u16)1 << 0,
    csFocus  = (u16)1 << 1,
  };
  using controlStateSet = u16;


  class control;

  enum controlAnchorSide
  {
    casBegin = 1,
    casEnd   = 2,
  };

  struct controlAnchor
  {
    bool Active;
    control *Control;
    controlAnchorSide Side = controlAnchorSide::casBegin;
  };

  struct controlAnchors
  {
    controlAnchor Left, Top, Righ, Bot;
  };


  enum controlMessages: u32
  {
    cmPaint = 1,
    cmReset = 2,

    cmMouseDown = 3,  // for Check
    cmMouseUp   = 4,  // for Check
    cmMouseMove = 5,  // for Check
  };


  class control
  {
    friend class view;

    public:
      control();
      virtual ~control();

    public:
      qsh<surface> Surface;
      poit_i32 Poit = {0, 0};
      size_i32 Size = {20, 10};
      poit_i32 EndPoit = {20,10};

      size_i32 PreferedSize = {0,0};
      size_i32 MinSize = {0,0};
      size_i32 MaxSize = {0,0};

      view *Parent = Nil;
      
      qsh<popup> Popup;

      bool AutoSize = false;
      bool Transparent = false;

      f64 Opacity = 1.0;

      string Name = "";
      vector<string> Style;
      vector<qsh<effect>> Effects;

      dirtyFlagSet DirtyFlags = (DirtyDraw | DirtyAutoSize | DirtyTiling);
      controlStateSet ControlState = 0;

      rect_i32 Margins = {0,0,0,0};
      rect_i32 Paddings = {0,0,0,0};
      controlAnchors Anchors = {{false, Nil},{false, Nil},{false, Nil},{false, Nil}};

      bool Visible = true;
      bool Enabled = true;

      u0 Tag = 0;
      jc_stc QDL = Nil;

      u64 LastClick = 0;


    public:
      virtual void Draw_before();
      virtual void Draw_after();
      virtual void Draw();

      virtual void CalcAutoSize();

      bool Show_Popup(poit_f32 Poit);

      control* GetRoot();
      void DyeToRoot(dirtyFlags Flag = DirtyDraw);

      inline bool CanAutoSizeVert() {return AutoSize && !(Anchors.Left.Active && Anchors.Righ.Active);}
      inline bool CanAutoSizeHorz() {return AutoSize && !(Anchors.Top.Active && Anchors.Bot.Active);}
      inline bool CanAutoSize() {return AutoSize && !((Anchors.Left.Active && Anchors.Righ.Active) || (Anchors.Top.Active && Anchors.Bot.Active));}

      void Flag_Add (dirtyFlagSet Flag);
      void Flag_Rem (dirtyFlagSet Flag);
      inline bool Flag_Has (dirtyFlagSet Flag) {return (DirtyFlags & Flag);}
      inline bool Flag_HasR(dirtyFlagSet Flag) {bool ret = (DirtyFlags & Flag); Flag_Rem(Flag); return ret;}


      virtual propError LoadProp(string Name, const jconf::Value& Prop);
      virtual bool LoadFunc(string Name, qev_seed FuncSeed);
      

    protected:
      bool _TC_Visited = false;
      bool _TC_Visiting = false;
      
      
    public:

      qev<> OnPaint;
      qev<> OnPaint_before;
      qev<> OnPaint_after;
      qev<> OnResize;
      
      qev<> OnClick;
      qev<poit_i32 /* Pos */> OnClickEx;
      qev<> OnDblClick;
      
      qev<poit_i32 /* Pos */, shiftStateSet /* Button */, shiftStateSet /* State */> OnMouseDown;
      qev<poit_i32 /* Pos */, shiftStateSet /* Button */, shiftStateSet /* State */> OnMouseUp;
      qev<poit_i32 /* Pos */, shiftStateSet /* State */> OnMouseMove;

      qev<char* /*Key*/, u32 /* KeyCode */, shiftStateSet /* State */> OnKeyDown;
      qev<char* /*Key*/, u32 /* KeyCode */, shiftStateSet /* State */> OnKeyUp;
      
      qev<poit_i32 /* Pos */, i16 /* Delta */, shiftStateSet /* State */ > OnScrollVert;
      qev<poit_i32 /* Pos */, i16 /* Delta */, shiftStateSet /* State */ > OnScrollHorz;
      
      qev<controlStateSet /* State */> OnStateChanged;


      virtual void Handler_Message     (controlMessages Msg);
      virtual void Handler_Paint       (); 
      virtual void Handler_Resize      ();
      virtual void Handler_MouseDown   (poit_i32 Pos, shiftStateSet Button, shiftStateSet State);
      virtual void Handler_MouseUp     (poit_i32 Pos, shiftStateSet Button, shiftStateSet State);
      virtual void Handler_MouseMove   (poit_i32 Pos, shiftStateSet State);
      virtual void Handler_KeyDown     (char *Key, u32 KeyCode, shiftStateSet State);
      virtual void Handler_KeyUp       (char *Key, u32 KeyCode, shiftStateSet State);
      virtual void Handler_ScrollVert  (poit_i32 Pos, i16 Delta, shiftStateSet State);
      virtual void Handler_ScrollHorz  (poit_i32 Pos, i16 Delta, shiftStateSet State);
      virtual void Handler_StateChanged(controlStateSet State);


      virtual void Do_Reset        ();
      virtual void Do_Paint_prepare();
      virtual void Do_Paint        ();
      virtual void Do_Resize       ();
      virtual void Do_Click        ();
      virtual void Do_ClickEx      (poit_i32 Pos);
      virtual void Do_DblClick     ();
      virtual void Do_MouseDown    (poit_i32 Pos, shiftStateSet Button, shiftStateSet State);
      virtual void Do_MouseUp      (poit_i32 Pos, shiftStateSet Button, shiftStateSet State);
      virtual void Do_MouseMove    (poit_i32 Pos, shiftStateSet State);
      virtual void Do_KeyDown      (char *Key, u32 KeyCode, shiftStateSet State);
      virtual void Do_KeyUp        (char *Key, u32 KeyCode, shiftStateSet State);
      virtual void Do_ScrollVert   (poit_i32 Pos, i16 Delta, shiftStateSet State);
      virtual void Do_ScrollHorz   (poit_i32 Pos, i16 Delta, shiftStateSet State);
      virtual void Do_StateChanged (controlStateSet State);
  };

}
