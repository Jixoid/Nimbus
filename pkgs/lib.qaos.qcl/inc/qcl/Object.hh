/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include <any>
#include <cassert>
#include <concepts>
#include <expected>
#include <functional>
#include <string_view>
#include <vector>

#include <sys/time.h>

#include "DS.hh"

#include "qcl/Types.hh"

using namespace std;


#define qcl_object
#define qcl_prop(...)
#define qcl_func


namespace qcl
{
  struct visual;    // Forward
  struct widget;    // Forward
  struct view;      // Forward


  /*
  enum capability: u16 {
    CapVisual    = (u16)1 << 1,
    CapTouchable = (u16)1 << 2,
    CapWritable  = (u16)1 << 3,
  };

  template <capability... Caps>
  struct __super
  {
    private:
      template<capability C> struct map;
      template<> struct map<capability::CapVisual>    { using type = qcl::visual; };
      template<> struct map<capability::CapTouchable> { using type = qcl::touchable; };
      template<> struct map<capability::CapWritable>  { using type = qcl::writable; };

      inline fun static consteval align_to(u0 addr, u0 align) -> u0 { return (addr + align - 1) & ~(align - 1); }

    private:
      std::tuple<typename map<Caps>::type*...> m_mods;

      inline constexpr __super() {
        ((std::get<typename map<Caps>::type*>(m_mods) = Nil), ...);
      }

    public:
      template <capability... ACaps>
      inline static fun makeNew() -> __super*
      {
        u0 size = sizeof(__super) + (sizeof(typename map<ACaps>::type)+...) + (alignof(typename map<ACaps>::type)+...);
        
        u8* ptr = (u8*)std::malloc(size);
        if (!ptr)
          throw std::bad_alloc();
        
        auto Ret = new ((void*)ptr) __super();
        
        u0 off = ((u0)ptr) + sizeof(__super);

        ([&] {
          using T = typename map<ACaps>::type;
          
          off = align_to(off, alignof(T));
          
          std::get<T*>(Ret->m_mods) = new ((void*)off) T();
          
          off += sizeof(T);
        }(), ...);

        return Ret;
      }

      template <capability... ACaps>
      inline static fun makeNew(typename map<ACaps>::type*... Args) -> __super*
      {
        u8* ptr = (u8*)std::malloc(sizeof(__super));
        if (!ptr)
          throw std::bad_alloc();
        
        auto Ret = new ((void*)ptr) __super();
        
        ([&] {
          using T = typename map<ACaps>::type;
          
          std::get<T*>(Ret->m_mods) = Args;
        }(), ...);

        return Ret;
      }

      
      inline void destroy() {
        ([&] {
          using T = typename map<Caps>::type;

          auto p = get<T*>();
          if (p)
            p->~T();
        }(), ...);

        m_mods->~decltype(m_mods)();
        
        std::free(this);
      }

      template <capability Cap>
      inline auto get() const noexcept -> typename map<Cap>::type*
      {
        return std::get<typename map<Cap>::type*>(m_mods);
      }

      template <capability Cap>
      inline auto has() const noexcept -> bool
      {
        return get<Cap>();
      }
  };

  using super = __super<CapVisual, CapTouchable, CapWritable>;
  */

  

  struct qcl_object object
  {
    public:
      friend struct view;

    public:
      inline constexpr object() {}
      inline constexpr virtual ~object() {}
      

    private:
      view *m_parent{};

      string m_name{};
      
      vector<string> m_styles{};

      ds::value m_qdl{ds::value::makeUnDef()};
      
      u0 m_tag{};

    public:
      inline fun parent() const noexcept -> view* {return m_parent;}
      fun setParent(view*) -> void;

      inline fun name() noexcept -> string {return m_name;}
      inline fun setName(string Val) noexcept -> void {m_name = Val;}
      qcl_prop(TYPE std::string, NAME name, READ name, WRITE setName);
      
      inline fun tag() const noexcept -> u0 {return m_tag;}
      inline fun setTag(u0 Val) noexcept -> void {m_tag = Val;}
      inline fun propTag(const ds::value& Prop) -> std::expected<bool, std::string> {
        if (!Prop.isInt()) return std::unexpected("was expected int");

        setTag(Prop.w_int());
        return true;
      }
      qcl_prop(TYPE u0, NAME tag, READ tag, WRITE setTag, PROP propTag);

      inline fun styles() -> vector<string>& {return m_styles;}

      inline fun qdl() -> ds::value {return m_qdl.nref();}
      inline fun setQDL(ds::value& Val) noexcept -> void {m_qdl = Val.nref();}

    protected:
      struct event_ {};

      template <bool Active, typename Type>
        requires std::derived_from<Type, event_>
      inline fun constexpr event_enabled(Type*& Inst) -> void {
        if constexpr (Active) {
          if (!Inst) Inst = new Type();
        }
        else {
          if (Inst) delete Inst; Inst = Nil;
        }
      }

    public:
      fun getRoot() noexcept -> view*;

    public:
      virtual fun getProp(string_view Name) -> expected<any, bool>;
      virtual fun setProp(string_view Name, any Prop) -> expected<void, bool>;
      virtual fun loadProp(string_view Name, const ds::value& Prop, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string>;
  };

}
