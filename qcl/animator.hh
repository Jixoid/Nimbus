/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


/**
 * @file animator.hh
 * @brief Animation system for visual elements.
 */

#pragma once

#include "qcl/basis.hh"
#include "qcl/object.hh"
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

  /**
   * @brief Linear interpolation function for animations.
   */
  template <typename T>
  fun lerp(types Beg, types End, f32 Time) -> types {
    f64 b = get<T>(Beg), e = get<T>(End);

    f64 res = b + (e - b) *Time;

    return T(res);
  }

  /**
   * @brief Ease in-out interpolation function for smoother animations.
   */
  template <typename T>
  fun easy(types Beg, types End, f32 t) -> types {
    f64 b = get<T>(Beg), e = get<T>(End);
    
    f32 smooth_t = t*t*t *(t *(t*6 -15) +10);

    f64 res = b + (e - b) * smooth_t;
    return T(res);
  }


  /**
   * @brief Structure representing a single animation node in the engine.
   */
  struct anim_node {
    u64 ID{};
    func Func{};

    visual* Object{};
    target_t Target{};
    
    types Beg{}, End{};
    u64 BegT{}, Interval{};
  };



  /**
   * @brief Active animation nodes list.
   */
  extern vector<anim_node> Anims;

  /**
   * @brief Pushes a new animation into the active list.
   * @return Animation ID.
   */
  fun AnimPush(anim_node) -> u64;

  /**
   * @brief Swaps or updates an existing animation.
   * @return Animation ID.
   */
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
      .Func = nil,
      .Object = Object,
      .Interval = Interval,
    };
    
    return AnimID == 0 ? AnimPush(Anim) : AnimSwap(Anim, AnimID);
  }

  /**
   * @brief Deletes an animation by ID.
   */
  fun AnimDel(u64) -> bool;


  /**
   * @brief Processes active animations. Called per frame.
   */
  fun AnimWorker() -> void;
}
