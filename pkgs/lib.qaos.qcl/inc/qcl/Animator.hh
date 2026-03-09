/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"
#include "Basis.hh"
#include "qcl/Object.hh"
#include <concepts>
#include <variant>

using namespace std;


namespace qcl::anim
{
  template <typename T> struct member_traits;

  template <typename C, typename A>
  struct member_traits<void (C::*)(A)> { 
    using arg_t = A; 
  };

  using types = variant<f32, f64, i32, i64, u32, u64>;
  using func = types (*)(types Beg, types End, f32 Time); 

  using target_t = variant<
    void (visual::*)(f32),
    void (visual::*)(f64),
    void (visual::*)(i32),
    void (visual::*)(i64),
    void (visual::*)(u32),
    void (visual::*)(u64)
  >;

  template <typename T>
  fun lerp(types Beg, types End, f32 Time) -> types {
    f64 b = get<T>(Beg), e = get<T>(End);

    f64 res = b + (e - b) *Time;

    return T(res);
  }

  template <typename T>
  fun easy(types Beg, types End, f32 t) -> types {
    f64 b = get<T>(Beg), e = get<T>(End);
    
    f32 smooth_t = t*t*t *(t *(t*6 -15) +10);

    f64 res = b + (e - b) * smooth_t;
    return T(res);
  }


  struct anim_node {
    u64 ID{};
    func Func{};

    visual* Object{};
    target_t Target{};
    
    types Beg{}, End{};
    u64 BegT{}, Interval{};
  };



  extern vector<anim_node> Anims;

  fun AnimPush(anim_node) -> u64;
  fun AnimSwap(anim_node, u64) -> u64;

  template <typename T, typename C>
    requires std::derived_from<C, visual>
  inline fun AnimSwap(visual* Object, void (C::*Prop)(T), func Func, T Beg, T End, u64 Interval, u64 AnimID = 0) -> u64
  {
    auto Anim = anim_node {
      .Func = Func,
      .Object = Object,
      .Target = (void (visual::*)(T))Prop,
      .Beg = Beg,
      .End = End,
      .Interval = Interval,
    };
    
    return AnimID == 0 ? AnimPush(Anim) : AnimSwap(Anim, AnimID);
  }
  
  inline fun AnimSwap(visual* Object, u64 Interval, u64 AnimID = 0) -> u64
  {
    auto Anim = anim_node {
      .Func = Nil,
      .Object = Object,
      .Interval = Interval,
    };
    
    return AnimID == 0 ? AnimPush(Anim) : AnimSwap(Anim, AnimID);
  }

  fun AnimDel(u64) -> bool;


  fun AnimWorker() -> void;
}
