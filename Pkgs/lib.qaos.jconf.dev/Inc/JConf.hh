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

#include <string>
#include <vector>
#include <stdexcept>
#include <initializer_list>

#include "Basis.h"
#include "JConf.h"


using namespace std;



namespace jconf
{

  class Value
  {
  public:
    // Constructors
    Value()
      : Handle(Nil)
      , IsOwner(true)
    {}

    explicit Value(jc_obj nHandle, bool TakeOwnership = true)
      : Handle(nHandle)
      , IsOwner(TakeOwnership)
    {}

    // Integer
    Value(int Val)
      : Handle(jc_new_int(Val))
      , IsOwner(true)
    {}

    Value(long Val)
      : Handle(jc_new_int(Val))
      , IsOwner(true)
    {}

    Value(long long Val)
      : Handle(jc_new_int(Val))
      , IsOwner(true)
    {}

    Value(unsigned long long Val)
      : Handle(jc_new_int(Val))
      , IsOwner(true)
    {}

    // Boolean
    Value(bool Val)
      : Handle(jc_new_bool(Val))
      , IsOwner(true)
    {}

    // Float
    Value(f64 Val)
      : Handle(jc_new_float(Val))
      , IsOwner(true)
    {}

    // String
    Value(const char* Val)
      : Handle(jc_new_val((char*)Val))
      , IsOwner(true)
    {}

    Value(const string& Val)
      : Handle(jc_new_val((char*)Val.c_str()))
      , IsOwner(true)
    {}


    // Binary Data
    Value(const data_64& Val)
      : Handle(jc_new_data(Val))
      , IsOwner(true)
    {}


    // Array (Initializer List)
    Value(initializer_list<Value> List)
    {
      vector<jc_obj> Objs;
      Objs.reserve(List.size());

      for(auto &Item: List)
        Objs.push_back(Item.getHandle());

      Handle = jc_new_arr(Objs.data(), Objs.size());
      IsOwner = true;
    }


    // Destructor
    ~Value()
    {
      if (IsOwner && Handle != Nil)
        jc_dis_obj(Handle);
    }

    // Copy Constructor
    Value(const Value& Other)
    {
      if (Other.Handle)
      {
        Handle = jc_copy(Other.Handle);
        IsOwner = true;
      } else {
        Handle = Nil;
        IsOwner = true;
      }
    }

    // Move Constructor
    Value(Value&& Other) noexcept
    {
      Handle = Other.Handle;
      IsOwner = Other.IsOwner;

      Other.Handle = Nil;
      Other.IsOwner = false;
    }

    // Copy Assignment
    Value& operator=(const Value& Other)
    {
      if (this == &Other)
        return *this;

      if (IsOwner && Handle)
        jc_dis_obj(Handle);


      if (Other.Handle) {
        Handle = jc_copy(Other.Handle);
        IsOwner = true;
      } else {
        Handle = Nil;
      }
      return *this;
    }

    // Move Assignment
    Value& operator=(Value&& Other) noexcept
    {
      if (this == &Other)
        return *this;

      if (IsOwner && Handle)
        jc_dis_obj(Handle);

      Handle = Other.Handle;
      IsOwner = Other.IsOwner;

      Other.Handle = Nil;
      Other.IsOwner = false;
      return *this;
    }


    // --- 3. Tip Kontrolleri ---
    bool isNull() const { return Handle == Nil; }
    bool isInt() const { return Handle && jc_is_int(Handle); }
    bool isBool() const { return Handle && jc_is_bool(Handle); }
    bool isFloat() const { return Handle && jc_is_float(Handle); }
    bool isString() const { return Handle && jc_is_val(Handle); }
    bool isData() const { return Handle && jc_is_data(Handle); }
    bool isStruct() const { return Handle && jc_is_stc(Handle); }
    bool isArray() const { return Handle && jc_is_arr(Handle); }


    // --- 4. Değer Okuma (Cast) ---
    operator int() const
    {  
      if (!isInt())
        throw runtime_error("Node is not integer");

      return (int)jc_int_get(Handle);
    }

    operator i64() const
    {
      if (!isInt())
        throw runtime_error("Node is not integer");
      
      return jc_int_get(Handle);
    }

    operator bool() const
    {
      if (!isBool())
        throw runtime_error("Node is not bool");
      
      return jc_bool_get(Handle);
    }

    operator f64() const
    {
      if (!isFloat())
        throw runtime_error("Node is not float");
      
      return jc_float_get(Handle);
    }

    operator string() const
    {
      if (!isString())
        throw runtime_error("Node is not string");

      const char* s = jc_val_get(Handle);
      return s ? string(s) : string();
    }

    operator data_64() const
    {
      if (!isData())
        throw runtime_error("Node is not data");

      return jc_data_get(Handle);
    }


    // --- 5. Struct & Array Erişimi ---

    // Struct Erişimi: config["key"]
    Value get(const string& Key) const
    {
      if (!isStruct())
        throw runtime_error("Node is not struct");

      // jc_stc_get yeni bir referans döndürür, Value bunu sahiplenir.
      return Value(jc_stc_get(Handle, Key.c_str()), true);
    }

    // Struct Atama: config.set("key", value)
    void set(const string& Key, const Value& Val)
    {
      if (!Handle) {
        // Eğer null ise otomatik Struct'a dönüştür (Kolaylık)
        Handle = jc_new_stc();
        IsOwner = true;
      }

      if (isStruct())
        jc_stc_set(Handle, Key.c_str(), Val.Handle);
    }

    // Array Erişimi: config[0]
    Value at(u32 Index) const
    {
      if (!isArray())
        return Value();

      return Value(jc_arr_get(Handle, Index), true);
    }

    void push(const Value& Val)
    {
      if (!Handle) {
        Handle = jc_new_arr(Nil, 0);
        IsOwner = true;
      }

      if (isArray())
        jc_arr_push(Handle, Val.Handle);
    }

    u32 size() const {
      if (isStruct()) return jc_stc_count(Handle);
      if (isArray()) return jc_arr_count(Handle);
      return 0;
    }

    // Operator [] (Sadece okuma kolaylığı için, yazma için .set() önerilir)
    Value operator[](const string& Key) const { return get(Key); }
    Value operator[](const char* Key) const { return get(Key); }
    Value operator[](int Index) const { return at(Index); }


    // --- 6. Dosya İşlemleri (Static) ---

    static Value Parse(const string& Path) {
      return Value(jc_parse(Path.c_str()), true);
    }

    static Value ParseRaw(const string& Path) {
      return Value(jc_parse_raw(Path.c_str()), true);
    }

    static Value ParseBin(const string& Path) {
      return Value(jc_parse_bin(Path.c_str()), true);
    }

    void saveRaw(const string& Path) const {
      if (isStruct())
        jc_write_raw(Path.c_str(), Handle);
    }

    void saveBin(const string& Path) const {
      if (isStruct())
        jc_write_bin(Path.c_str(), Handle);
    }

    // Internal Access
    jc_obj getHandle() const { return Handle; }

  private:
    jc_obj Handle;
    bool IsOwner;
  };

}
