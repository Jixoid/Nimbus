/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <vector>

#include "Basis.h"

#include "qcl/Control.hh"
#include "qcl/Graphic.hh"
#include "JConf.hh"

using namespace std;
using namespace jix;



namespace qcl
{

  class view: public control
  {
    public:
      view();
      ~view() override;


    public:
      vector<qsh<control>> Childs;

      control
        *FHoverControl = Nil,
        *FFocusControl = Nil;

    private:
      bool FIn = false;
      control
        *FLegHoverControl = Nil,
        *FLegFocusControl = Nil;
    
      void HoverControlSet(control* Ctrl);
      void FocusControlSet(control* Ctrl);

    public:
      bool ScrollVertVisible = true;
      bool ScrollHorzVisible = true;

      void Child_Add(qsh<control> Ctrl);
      void Child_Rem(control* Ctrl);

    public:
      rect_i32 ClientBound = {0,0,0,0};
      poit_i32 ClientPos = {0,0};

      bool ScrollableVert = false;
      bool ScrollableHorz = false;


    public:
      void Draw_after() override;
      void CalcAutoSize() override;

      virtual void Draw_ScrollVert();
      virtual void Draw_ScrollHorz();

      propError LoadProp(string Name, const jconf::Value& Prop) override;

    public:
      virtual control* FindInput(poit_i32 Pos);

      void TilingScanH(vector<control*> &HList, control *This);
      void TilingScanV(vector<control*> &HList, control *This);

      virtual void Do_Tiling();

      void Handler_StateChanged(controlStateSet State) override;

      void Do_Reset        () override;
      void Do_Paint_prepare() override;
      void Do_Paint        () override;
      void Do_Resize       () override;
      void Do_MouseDown    (poit_i32 Pos, shiftStateSet Button, shiftStateSet State) override;
      void Do_MouseUp      (poit_i32 Pos, shiftStateSet Button, shiftStateSet State) override;
      void Do_MouseMove    (poit_i32 Pos, shiftStateSet State) override;
      void Do_KeyDown      (char *Key, u32 KeyCode, shiftStateSet State) override;
      void Do_KeyUp        (char *Key, u32 KeyCode, shiftStateSet State) override;
      void Do_ScrollVert   (poit_i32 Pos, i16 Delta, shiftStateSet State) override;
      void Do_ScrollHorz   (poit_i32 Pos, i16 Delta, shiftStateSet State) override;
  };

}
