/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"
#include "qcl/Types.hh"

using namespace jix;



namespace qcl
{
  class window;


  // Surface
  class surface
  {
    public:

    enum operators: u16
    {
      soNormal = 0,
      soClear  = 1,
      soIn     = 2,
    };


    public:
      surface(u32 nWidth, u32 nHeight);
      surface(u32 nWidth, u32 nHeight, u32 *Data);
      surface(handle nOHID);
      
      static surface* FromFile_SVG(u32 nWidth, u32 nHeight, const char* Path);
      static surface* FromData_SVG(u32 nWidth, u32 nHeight, const u8* Data, u0 DSize);
      
      ~surface();

    public:
      handle OHID;


    public:
      u32 Width, Height;

      f32 FontSize;
      f32 LineSize;

    public:
      void Set_Color(color Val);
      void Set_Source(qsh<surface> Val);
      void Set_SourceP(surface *Val);
      void Set_FontSize(f32 Size);
      void Set_LineSize(f32 Size);

      void Set_Translate(poit_f32 P);

      void      Set_OP(operators Op);
      operators Get_OP();

      void Set_Pos(poit_f32 Val);
      void Set_rPos(poit_f32 Val);
      void Set_Size(u32 nWidth, u32 nHeight);

      void Set_Dirty();
      u32* Get_Data();

    public:
      void Draw_Rect(rect_f32 Area);
      void Draw_RectRound(rect_f32 Area, f32 Radius);
      void Draw_RectRound(rect_f32 Area, f32 R_LT, f32 R_LB, f32 R_RT, f32 R_RB);
      void Draw_Line(poit_f32 P1, poit_f32 P2);
      void Draw_ToLine(poit_f32 P);
      void Draw_Arc(poit_f32 O, f32 Radius, f32 Ang1, f32 Ang2);
      void Draw_Text(const char* Text);

      size_f32 Calc_Text(const char* Text);

    public:
      void Filter_GrayScale(rect_f32 Area);
      
    
    public:
      void Connect(qcl::window *PlatformWindow);

      void Stroke();
      void Fill();
      void Paint();
      void PaintA(f64 A);
      void Clip();
      void Clip_Reset();

      void Sets_Push();
      void Sets_Pop();

      void Clear();
      void Flush();
  };

}
