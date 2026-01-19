/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#include "Basis.h"

#include "qcl/Graphic.hh"
#include "qcl/Window.hh"
#include "qcl/Platform.hh"

using namespace std;
using namespace jix;


namespace qcl
{

  surface::surface(u32 nWidth, u32 nHeight)
  {
    platform::surface::New(&OHID, nWidth, nHeight);
  }

  surface::surface(u32 nWidth, u32 nHeight, u32 *Data)
  {
    platform::surface::New_Data(&OHID, nWidth, nHeight, Data);
  }

  surface::surface(handle nOHID)
    : OHID(nOHID)
  {}

  surface* surface::FromFile_SVG(u32 nWidth, u32 nHeight, const char* Path)
  {
    ohid nOHID;

    platform::surface::New_File_SVG(&nOHID, nWidth, nHeight, Path);

    if (nOHID == 0)
      throw runtime_error("SVG not loaded");

    auto Sur = new surface(nOHID);

    return std::move(Sur);
  }

  surface* surface::FromData_SVG(u32 nWidth, u32 nHeight, const u8* Data, u0 DSize)
  {
    ohid nOHID;

    platform::surface::New_Data_SVG(&nOHID, nWidth, nHeight, Data, DSize);

    if (nOHID == 0)
      throw runtime_error("SVG not loaded");

    auto Sur = new surface(nOHID);

    return std::move(Sur);
  }

  surface::~surface()
  {
    platform::surface::Dis(OHID);
  }


  void surface::Set_Color(color Val)
  {
    platform::surface::Set_Color(OHID, Val);
  }

  void surface::Set_Source(qsh<surface> Val)
  {
    platform::surface::Set_Source(OHID, Val.get());
  }

  void surface::Set_SourceP(surface *Val)
  {
    platform::surface::Set_Source(OHID, Val);
  }

  void surface::Set_FontSize(f32 Size)
  {
    FontSize = Size;
    platform::surface::Set_FontSize(OHID, Size);
  }

  void surface::Set_LineSize(f32 Size)
  {
    LineSize = Size;
    platform::surface::Set_LineSize(OHID, Size);
  }

  void surface::Set_Translate(poit_f32 P)
  {
    platform::surface::Set_Translate(OHID, P);
  }


  void surface::Set_OP(operators Op)
  {
    platform::surface::Set_OP(OHID, Op);
  }
    
  surface::operators surface::Get_OP()
  {
    return platform::surface::Get_OP(OHID);
  }


  void surface::Set_Pos(poit_f32 Val)
  {
    platform::surface::Set_Pos(OHID, Val);
  }

  void surface::Set_rPos(poit_f32 Val)
  {
    platform::surface::Set_rPos(OHID, Val);
  }

  void surface::Set_Size(u32 nWidth, u32 nHeight)
  {
    platform::surface::Set_Size(OHID, nWidth, nHeight);

    Width  = nWidth;
    Height = nHeight;
  }


  void surface::Set_Dirty()
  {
    platform::surface::Set_Dirty(OHID);
  }
  
  u32* surface::Get_Data()
  {
    u32 *Ret;

    platform::surface::Get_Data(OHID, &Ret);

    return Ret;
  }

  
  void surface::Draw_Rect(rect_f32 Area)
  {
    platform::surface::Draw_Rect(OHID, Area);
  }

  void surface::Draw_RectRound(rect_f32 Area, f32 Radius)
  {
    platform::surface::Draw_RectRound(OHID, Area, Radius);
  }

  void surface::Draw_RectRound(rect_f32 Area, f32 R_LT, f32 R_LB, f32 R_RT, f32 R_RB)
  {
    platform::surface::Draw_RectRound4(OHID, Area, R_LT, R_LB, R_RT, R_RB);
  }

  void surface::Draw_Line(poit_f32 P1, poit_f32 P2)
  {
    platform::surface::Draw_Line(OHID, P1, P2);
  }

  void surface::Draw_ToLine(poit_f32 P)
  {
    platform::surface::Draw_ToLine(OHID, P);
  }

  void surface::Draw_Arc(poit_f32 O, f32 Radius, f32 Ang1, f32 Ang2)
  {
    platform::surface::Draw_Arc(OHID, O, Radius, Ang1, Ang2);
  }

  void surface::Draw_Text(const char* Text)
  {
    platform::surface::Draw_Text(OHID, Text);
  }

  size_f32 surface::Calc_Text(const char* Text)
  {
    size_f32 Ret;

    platform::surface::Calc_Text(OHID, Text, &Ret);

    return Ret;
  }


  void surface::Filter_GrayScale(rect_f32 Area)
  {
    platform::surface::Filter_GrayScale(OHID, Area);
  }


  void surface::Connect(qcl::window *Window)
  {
    platform::surface::Connect(OHID, Window->OHID);
  }
  
  void surface::Stroke()
  {
    platform::surface::Stroke(OHID);
  }

  void surface::Fill()
  {
    platform::surface::Fill(OHID);
  }

  void surface::Paint()
  {
    platform::surface::Paint(OHID);
  }

  void surface::PaintA(f64 A)
  {
    platform::surface::PaintA(OHID, A);
  }

  void surface::Clip()
  {
    platform::surface::Clip(OHID);
  }
  
  void surface::Clip_Reset()
  {
    platform::surface::Clip_Reset(OHID);
  }


  void surface::Sets_Push()
  {
    platform::surface::Sets_Push(OHID);
  }

  void surface::Sets_Pop()
  {
    platform::surface::Sets_Pop(OHID);
  }


  void surface::Clear()
  {
    platform::surface::Clear(OHID);
  }

  void surface::Flush()
  {
    platform::surface::Flush(OHID);
  }

}