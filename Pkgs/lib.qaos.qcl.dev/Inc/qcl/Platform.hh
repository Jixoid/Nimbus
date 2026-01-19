/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"

#include "qcl/Control.hh"
#include "qcl/Graphic.hh"

using namespace std;
using namespace jix;



namespace qcl::platform
{
  extern "C" [[gnu::weak]] int qcl_entry1(ohid App_H, ohid Win_H);
  extern "C" [[gnu::weak]] int qcl_entry2(ohid App_H);
  extern "C" [[gnu::weak]] int qcl_entry3();
  
  extern "C" [[gnu::noreturn]] void qcl_error(const char *Msg);

  
  namespace window
  {
    void New(handle *ret);
    void Dis(handle  val);

    void Show(handle val);
    void Hide(handle val);

    void     SizeSet(handle val, size_i32 Size);
    size_i32 SizeGet(handle val);

    void StartMove(handle val, poit_i32 Pos);
  }

  namespace surface
  {
    void New(handle *ret, u32 nWidth, u32 nHeight);
    void New_Data(handle *ret, u32 nWidth, u32 nHeight, u32 *Data);
    void New_File_SVG(handle *ret, u32 nWidth, u32 nHeight, const char* Path);
    void New_Data_SVG(handle *ret, u32 nWidth, u32 nHeight, const u8* Data, u0 DSize);
    void Dis(handle  val);


    void Set_Color(handle  val, color Val);
    void Set_Source(handle  val, qcl::surface *Val);
    void Set_FontSize(handle  val, f32 Size);
    void Set_LineSize(handle  val, f32 Size);

    void Set_Translate(handle  val, poit_f32 P);

    void Set_OP(handle  val, qcl::surface::operators  Op);
    qcl::surface::operators Get_OP(handle  val);

    void Set_Pos(handle  val, poit_f32 Val);
    void Set_rPos(handle  val, poit_f32 Val);
    void Set_Size(handle  val, u32 nWidth, u32 nHeight);

    void Set_Dirty(handle  val);
    void Get_Data(handle  val, u32 **Data);

    void Draw_Rect(handle  val, rect_f32 Area);
    void Draw_RectRound(handle  val, rect_f32 Area, f32 Radius);
    void Draw_RectRound4(handle  val, rect_f32 Area, f32 R_LT, f32 R_LB, f32 R_RT, f32 R_RB);
    void Draw_Line(handle  val, poit_f32 P1, poit_f32 P2);
    void Draw_ToLine(handle  val, poit_f32 P);
    void Draw_Arc(handle  val, poit_f32 O, f32 Radius, f32 Ang1, f32 Ang2);
    void Draw_Text(handle  val, const char* Text);

    void Calc_Text(handle  val, const char* Text, size_f32 *Size);

    void Filter_GrayScale(handle  val, rect_f32 Area);

    void Connect(handle  val, handle Window);

    void Stroke(handle  val);
    void Fill(handle  val);
    void Paint(handle  val);
    void PaintA(handle  val, f64 A);
    void Clip(handle  val);
    void Clip_Reset(handle  val);

    void Sets_Push(handle  val);
    void Sets_Pop(handle  val);

    void Clear(handle  val);
    void Flush(handle  val);
  }

  namespace opengl
  {
    void New(handle *ret, u32 nWidth, u32 nHeight);
    void Dis(handle  val);

    void Set_Current(handle  val);
    void Set_Size(handle  val, u32 nWidth, u32 nHeight);

    void ReadBuffer(handle  val, u32 *PixBuf);

    void Flush(handle  val);
  }

  namespace application
  {
    void New(handle *ret);
    void Dis(handle  val);


    void PushMessage(handle val, qcl::control* Ctrl, controlMessages Msg);
    void PushTask(handle val, void (*Fun)(u0), u0 Data);

    bool CheckQueue(handle val, qcl::control* Ctrl, controlMessages Msg);

    void Run(handle val);
  }

  namespace dialog
  {
    char* FileOpen(const char *Title, const char *Filters[]);
    char* FileSave(const char *Title, const char *Filters[]);
    void  Message(const char *Text);
  }

}
