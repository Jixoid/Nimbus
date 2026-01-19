/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"
#include <concepts>
#include <memory>
#include <type_traits>
#include <utility>

using namespace jix;

using namespace std;



namespace qcl
{
  class window;


  // Color
  struct color
  {
    f32
      A,R,G,B;

    inline color(f32 nR, f32 nG, f32 nB, f32 nA = 1.0f)
      : R(nR)
      , G(nG)
      , B(nB)
      , A(nA)
    {}

    inline color _A(f32 nA)
    {
      color Ret = *this;
      Ret.A = nA;

      return Ret;
    }

    inline color _R(f32 nR)
    {
      color Ret = *this;
      Ret.R = nR;

      return Ret;
    }
    
    inline color _G(f32 nG)
    {
      color Ret = *this;
      Ret.G = nG;

      return Ret;
    }

    inline color _B(f32 nB)
    {
      color Ret = *this;
      Ret.B = nB;

      return Ret;
    }


    inline bool operator==(const color &It) const
    {
      return (A == It.A) && (R == It.R) && (G == It.G) && (B == It.B);
    }

  };



  // Rect
  template <typename _T>
  struct rect
  {
    _T
      X1,Y1,
      X2,Y2;

    inline bool operator==(const rect &It) const
    {
      return (X1 == It.X1) && (X2 == It.X2) && (Y1 == It.Y1) && (Y2 == It.Y2);
    }
  };

  using rect_f32 = rect<f32>;
  using rect_i32 = rect<i32>;



  // Poit
  template <typename _T>
  struct poit
  {
    _T X,Y;

    inline bool operator==(const poit &It) const
    {
      return (X == It.X) && (Y == It.Y);
    }

    inline poit operator+(const poit &It) const
    {
      return {X +It.X, Y +It.Y};
    }

    inline poit& operator+=(const poit &It)
    {
      X += It.X;
      Y += It.Y;

      return *this;
    }
  };

  using poit_f32 = poit<f32>;
  using poit_i32 = poit<i32>;



  // Size
  template <typename _T>
  struct size
  {
    _T W,H;

    inline bool operator==(const size &It) const
    {
      return (W == It.W) && (H == It.H);
    }
  };

  using size_f32 = size<f32>;
  using size_i32 = size<i32>;



  // QEV
  class control;
  class view;

  template <typename... args>
  struct qev
  {
    view *Inst{};
    void (view::*Func)(control*, args...){};


    qev(){}

    qev(view *nInst, void (view::*nFunc)(control*, args...))
      : Inst(nInst)
      , Func(nFunc)
    {}


    inline void Call(control *Ctrl, args... Args)
    {
      if (Inst && Func)
        (Inst->*Func)(Ctrl, std::forward<args>(Args)...);
    }

    explicit operator bool() const
    {
      return (Inst && Func);
    }
  };


  template <typename T>
  struct is_qev : std::false_type {};

  template <typename... Args>
  struct is_qev<qev<Args...>> : std::true_type {};


  template <typename T>
  concept qev_ct = is_qev<T>::value;

  struct qev_seed
  {
    view *Inst{};
    void (view::*Func)(){};
    
    
    qev_seed(){}

    template <typename... args>
    qev_seed(const qev<args...> &Other) 
      : Inst(Other.Inst)
      , Func((void (view::*)())Other.Func)
    {}


    template <typename T, typename... args>
      requires std::derived_from<T, view>
    qev_seed(T *nInst, void (T::*nFunc)(control*, args...))
      : Inst((view*)nInst)
      , Func((void (view::*)())nFunc)
    {}


    template <typename... args>
    void ToLoad(qev<args...> &Target) const
    {
      Target.Inst = this->Inst;
      Target.Func = (void (view::*)(control*, args...))this->Func;
    }
  };


  // QSH
  template <typename T>
  using qsh = __shared_ptr<T, _Lock_policy::_S_single>;
}
