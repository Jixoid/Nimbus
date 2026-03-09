/*
 *  This file is part of QAOS
 *
 *  This file is licensed under the GNU General Public License version 3 (GPL3).
 *
 *  You should have received a copy of the GNU General Public License
 *  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 *
 *  Copyright (c) 2025 by Kadir Aydın.
 */


#pragma once

#include <algorithm>
#include <cassert>
#include <cstring>
#include <iterator>
#include <ostream>
#include <span>
#include <string>
#include <string_view>
#include <stdexcept>
#include <vector>

#include "Basis.h"
#include "Basis.hh"


using namespace std;



namespace qcl::ds
{
  enum etype: u8 {
    etNull = 0,

    etStr = 1,
    etArr = 2,
    etStc = 3,
    etTup = 4,
    etTyp = 5,
    etRef = 6,
    etI64 = 7,
    etF64 = 8,
    etBoo = 9,
  };


  namespace {
    struct payload {
      void* ptr{};
      etype type{};
      bool view{};
    };
  }

  struct type_str  { string val{}; };
  struct type_vstr { string_view val{}; };
  
  struct type_arr  { vector<sptr<payload>> reals{}; };
  struct type_varr { span<void*> vals{}; span<etype> types{}; };
  
  struct type_stc:  type_arr  { vector<string> names{}; };
  struct type_vstc: type_varr { span<string_view> names{}; };
  
  struct type_tup:  type_arr  {};
  struct type_vtup: type_varr {};
  
  struct type_typ  { pair<string, string> val{}; };
  struct type_vtyp { string_view typ{}, val{}; };
  
  union type_val  { i64 I; f64 F; bool B; };
  union type_vval { i64* I; f64* F; bool* B; };
  
  struct type_ref  { string val{}; };
  struct type_vref { string_view val{}; };



  namespace
  {
    inline fun type_free(payload Payload) -> void {
      auto [ptr,type,span] = Payload;
      
      switch (type) {
        case etNull: break;
        
        case etStr: span ? delete (type_vstr*)ptr : delete (type_str*)ptr; break;
        case etArr: span ? delete (type_varr*)ptr : delete (type_arr*)ptr; break;
        case etStc: span ? delete (type_vstc*)ptr : delete (type_stc*)ptr; break;
        case etTup: span ? delete (type_vtup*)ptr : delete (type_tup*)ptr; break;
        case etTyp: span ? delete (type_vtyp*)ptr : delete (type_typ*)ptr; break;
        case etRef: span ? delete (type_vref*)ptr : delete (type_ref*)ptr; break;
        
        case etI64:
        case etF64:
        case etBoo: span ? delete (type_vval*)ptr : delete (type_val*)ptr; break;

        default: throw runtime_error("unknown qcl::ds type");
      }
    }
    

    inline fun create_sptr(payload Payload) -> sptr<payload> {
      return sptr<payload>(new payload(Payload), [](payload* res){ type_free(*res); delete res; });
    }
  }

  

  struct value
  {
    public:
      value() = delete;

      inline constexpr value(sptr<payload> Payload) : m_real(std::move(Payload)) {}

      inline constexpr value(value&& it) : m_real(std::move(it.m_real)) {}

      inline constexpr fun operator=(value&& it) noexcept -> value& {
        if (this != &it)
          m_real = std::move(it.m_real);

        return *this;
      }

      value(const value&) = delete;
      fun operator=(const value&) -> value& = delete;

    public:
      [[nodiscard]] static fun makeFromStream(istream*) -> value;
      [[nodiscard]] static fun makeFromStreamRaw(istream*) -> value;
      [[nodiscard]] static fun makeFromStreamBin(istream*) -> value;

      static fun saveToStreamRaw(ostream*, value&) -> void;
      static fun saveToStreamBin(ostream*, value&) -> void;

    public:
      [[nodiscard]] static fun makeCopy(const value&) -> value;
      [[nodiscard]] static fun makeNRef(const value&) -> value;

      [[nodiscard]] inline fun copy() -> value { return value::makeCopy(*this); }
      [[nodiscard]] inline fun nref() -> value { return value::makeNRef(*this); }

      inline fun saveRaw(ostream*) -> void;
      inline fun saveBin(ostream*) -> void;

    public:
      [[nodiscard]] inline constexpr static fun makeNull() -> value {
        return value(create_sptr({Nil, etype::etNull, false}));
      }
      [[nodiscard]] inline constexpr static fun makeUnDef() -> value {
        return value(sptr<payload>(Nil));
      }


      [[nodiscard]] inline constexpr static fun makeString(string_view val) -> value {
        return value(create_sptr({new type_str{string(val)}, etype::etStr, false}));
      }
      [[nodiscard]] inline constexpr static fun makeArray() -> value {
        return value(create_sptr({new type_arr{}, etype::etArr, false}));
      }
      [[nodiscard]] inline constexpr static fun makeStruct() -> value {
        return value(create_sptr({new type_stc{}, etype::etStc, false}));
      }
      [[nodiscard]] inline constexpr static fun makeTuple() -> value {
        return value(create_sptr({new type_tup{}, etype::etTup, false}));
      }
      [[nodiscard]] inline constexpr static fun makeType(string_view typ, string_view val) -> value {
        return value(create_sptr({new type_typ{{string(typ),string(val)}}, etype::etTyp, false}));
      }
      [[nodiscard]] inline constexpr static fun makeRef(string_view val) -> value {
        return value(create_sptr({new type_ref{string(val)}, etype::etRef, false}));
      }
      [[nodiscard]] inline constexpr static fun makeI64(i64 val) -> value {
        return value(create_sptr({new type_val{.I = val}, etype::etI64, false}));
      }
      [[nodiscard]] inline constexpr static fun makeF64(f64 val) -> value {
        return value(create_sptr({new type_val{.F = val}, etype::etF64, false}));
      }
      [[nodiscard]] inline constexpr static fun makeBool(bool val) -> value {
        return value(create_sptr({new type_val{.B = val}, etype::etBoo, false}));
      }
      
      [[nodiscard]] inline constexpr static fun makeVString(string_view val) -> value {
        return value(create_sptr({new type_vstr{val}, etype::etStr, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVArray(span<void*> vals, span<etype> types) -> value {
        return value(create_sptr({new type_varr{vals, types}, etype::etArr, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVStruct(span<void*> vals, span<etype> types, span<string_view> names) -> value {
        return value(create_sptr({new type_vstc{vals, types, names}, etype::etStc, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVTuple(span<void*> vals, span<etype> types) -> value {
        return value(create_sptr({new type_vtup{vals, types}, etype::etTup, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVType(string_view typ, string_view val) -> value {
        return value(create_sptr({new type_vtyp{typ,val}, etype::etTyp, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVRef(string_view val) -> value {
        return value(create_sptr({new type_vref{val}, etype::etRef, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVI64(i64* val) -> value {
        return value(create_sptr({new type_vval{.I = val}, etype::etI64, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVF64(f64* val) -> value {
        return value(create_sptr({new type_vval{.F = val}, etype::etF64, true}));
      }
      [[nodiscard]] inline constexpr static fun makeVBool(bool* val) -> value {
        return value(create_sptr({new type_vval{.B = val}, etype::etBoo, true}));
      }

    private:
      sptr<payload> m_real{};

    public:
      inline fun real() { return m_real; }
    
    public:
      inline fun isUnDef() const noexcept -> bool { return !m_real; }
      inline fun isNull() const noexcept -> bool  { return m_real && (m_real->type == etype::etNull); }

      inline fun isString() const noexcept -> bool { return m_real && (m_real->type == etype::etStr); }
      inline fun isArray() const noexcept -> bool  { return m_real && (m_real->type == etype::etArr); }
      inline fun isStruct() const noexcept -> bool { return m_real && (m_real->type == etype::etStc); }
      inline fun isTuple() const noexcept -> bool  { return m_real && (m_real->type == etype::etTup); }
      inline fun isType() const noexcept -> bool   { return m_real && (m_real->type == etype::etTyp); }
      inline fun isRef() const noexcept -> bool    { return m_real && (m_real->type == etype::etRef); }
      inline fun isInt() const noexcept -> bool    { return m_real && (m_real->type == etype::etI64); }
      inline fun isFloat() const noexcept -> bool  { return m_real && (m_real->type == etype::etF64); }
      inline fun isBool() const noexcept -> bool   { return m_real && (m_real->type == etype::etBoo); }

      inline fun isView() const noexcept -> bool { assert(m_real); return m_real->view; }

    public:
      inline fun w_string() const noexcept -> string { assert(isString()); return isView() ? string(((type_vstr*)m_real->ptr)->val) : ((type_str*)m_real->ptr)->val; }
      inline fun w_int() const noexcept -> i64       { assert(isInt());    return isView() ? *((type_vval*)m_real->ptr)->I : ((type_val*)m_real->ptr)->I; }
      inline fun w_float() const noexcept -> f64     { assert(isFloat());  return isView() ? *((type_vval*)m_real->ptr)->F : ((type_val*)m_real->ptr)->F; }
      inline fun w_bool() const noexcept -> bool     { assert(isBool());   return isView() ? *((type_vval*)m_real->ptr)->B : ((type_val*)m_real->ptr)->B; }
      
      inline fun w_ref() const noexcept -> string    { assert(isRef());    return isView() ? string(((type_vref*)m_real->ptr)->val) : ((type_ref*)m_real->ptr)->val; }

      inline fun w_type() const noexcept -> pair<string,string>
      {
        assert(isType());
        
        return isView()
          ? pair{string(((type_vtyp*)m_real->ptr)->typ), string(((type_vtyp*)m_real->ptr)->val)}
          : ((type_typ*)m_real->ptr)->val;
      }

    public:
      inline fun size() const noexcept -> u32 { assert(isArray() || isStruct()); return isView() ? ((type_varr*)m_real->ptr)->vals.size() : ((type_arr*)m_real->ptr)->reals.size(); }

      inline fun contains(etype val) const noexcept -> bool
      {
        assert(isTuple());
        
        if (isView()) {
          for (const auto X: ((type_vtup*)m_real->ptr)->types)
            if (val == X)
              return true;

          return false;
        }
        else {
          for (const auto X: ((type_tup*)m_real->ptr)->reals)
            if (val == X->type)
              return true;

          return false;
        }
      }

    public:
      inline fun push_back(value&& val) -> void
      {
        assert((isArray() || isTuple()) && !isView());

        auto real = ((type_arr*)m_real->ptr);

        real->reals.push_back(std::move(val.m_real));
      }

      inline fun push_back(value&& val, string_view key) -> void
      {
        assert(isStruct() && !isView());

        auto real = ((type_stc*)m_real->ptr);

        real->reals.push_back(std::move(val.m_real));
        real->names.push_back(string(key));
      }

      inline fun push_back(value&& val, etype key) -> void
      {
        assert(isTuple() && !isView());

        auto real = ((type_tup*)m_real->ptr);

        real->reals.push_back(std::move(val.m_real));
      }

      inline fun get_stc(u32 index) const -> pair<string, value>
      {
        assert(isStruct());

        if (index >= size()) return {"", value::makeUnDef()};
        
        auto real = isView()
          ? create_sptr({
              ((type_vstc*)m_real->ptr)->vals[index],
              ((type_vstc*)m_real->ptr)->types[index],
              true
            })
          : ((type_stc*)m_real->ptr)->reals[index];

        auto key = isView()
          ? string(((type_vstc*)m_real->ptr)->names[index])
          : ((type_stc*)m_real->ptr)->names[index];

        return {key, value(real)};
      }


    public:
      [[nodiscard]] inline fun operator[](string_view key) const -> value
      {
        assert(isStruct());

        auto idx = isView()
          ? [key](span<string_view> names) -> i64 {
              auto it = std::find(names.begin(), names.end(), key);

              return (it != names.end()) ? std::distance(names.begin(), it) : -1;
            }(((type_vstc*)m_real->ptr)->names)
          : [key](const vector<string>& names) -> i64 {
              auto it = std::find(names.begin(), names.end(), key);

              return (it != names.end()) ? std::distance(names.begin(), it) : -1;
            }(((type_stc*)m_real->ptr)->names);


        if (idx < 0)
          return value::makeUnDef();

        return (*this)[idx];
      }

      value operator[](const char* key) const { return (*this)[string_view(key, strlen(key))]; }

      inline fun operator[](u32 index) const -> value
      {
        assert(isArray() || isStruct());

        if (index >= size()) return value::makeUnDef();
        
        auto real = isView()
          ? sptr<payload>(create_sptr({
              ((type_varr*)m_real->ptr)->vals[index],
              ((type_varr*)m_real->ptr)->types[index],
              false,
            }))
          : ((type_arr*)m_real->ptr)->reals[index];

        return value(real);
      }

      inline fun operator[](etype val) const -> value
      {
        assert(isTuple());

        if (!contains(val)) return value::makeUnDef();

        u0 idx = isView()
          ? [this, val](){
              for (u0 i{}; i < ((type_vtup*)m_real->ptr)->types.size(); i++)
                if (val == ((type_vtup*)m_real->ptr)->types[i])
                  return i;

              throw runtime_error("internal error");
            }()
          : [this, val](){
              for (u0 i{}; i < ((type_tup*)m_real->ptr)->reals.size(); i++)
                if (val == ((type_tup*)m_real->ptr)->reals[i]->type)
                  return i;

              throw runtime_error("internal error");
            }();
        
        auto real = isView()
          ? sptr<payload>(create_sptr({
              ((type_vtup*)m_real->ptr)->vals[idx],
              ((type_vtup*)m_real->ptr)->types[idx],
              true,
            }))
          : ((type_tup*)m_real->ptr)->reals[idx];

        return value(real);
      }

  };

}
