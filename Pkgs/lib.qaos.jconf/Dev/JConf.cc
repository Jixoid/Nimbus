/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025 by Kadir Aydın.
*/



#define el  else
#define ef  else if

#include <iostream>
#include <vector>
#include <unordered_map>
#include <fstream>
#include <string.h>
#include <memory>
#include <algorithm>
#include <utility>
#include <charconv>
#include <cstdlib>
#include <iomanip>

#include "Basis.h"
#include "JConf.h"

using namespace std;



// Magic
const char _c_Mag_Jixoid[10] = {'\e','^','J','I','X','0','1','D','!','\a'};

const char _c_Mag_JConf[7] = {'C','o','n','f','i','g','_'};


const u8 _c_Val   = 0x1;
const u8 _c_Stc   = 0x2;
const u8 _c_Arr   = 0x3;
const u8 _c_Int   = 0x4;
const u8 _c_Bool  = 0x5;
const u8 _c_Data  = 0x6;
const u8 _c_Float = 0x7;


// Intern
template <typename T>
using jc = shared_ptr<T>;


struct cObj
{
  cObj(u8 nType)
    : Type(nType)
  {}

  u8 Type;
};


struct cVal: cObj
{
  cVal() : cObj(_c_Val)
  {}

  cVal(string nVal)
    : cObj(_c_Val)
    , Val(nVal)
  {}

  string Val;
};

struct cInt: cObj
{
  cInt() : cObj(_c_Int)
  {}

  cInt(i64 nVal)
    : cObj(_c_Int)
    , Val(nVal)
  {}

  i64 Val;
};

struct cBool: cObj
{
  cBool() : cObj(_c_Bool)
  {}

  cBool(bool nVal)
    : cObj(_c_Bool)
    , Val(nVal)
  {}

  bool Val;
};

struct cFloat: cObj
{
  cFloat() : cObj(_c_Float)
  {}

  cFloat(f64 nVal)
    : cObj(_c_Float)
    , Val(nVal)
  {}

  f64 Val;
};

struct cData: cObj
{
  cData()
    : cObj(_c_Data)
    , Val({Nil, 0})
  {}

  cData(data_64 nVal)
    : cObj(_c_Data)
    , Val(nVal)
  {}


  ~cData()
  {
    if (Val.Point != Nil)
      free(Val.Point);
  }

  data_64 Val;
};

struct cStc: cObj
{
  cStc() : cObj(_c_Stc)
  {}

  vector<pair<string, jc<cObj>>> Stc;
};

struct cArr: cObj
{
  cArr() : cObj(_c_Arr)
  {}

  cArr(vector<jc<cObj>> nArr)
    : cObj(_c_Arr)
    , Arr(nArr)
  {}

  vector<jc<cObj>> Arr;
};




thread_local u32 JC_ERROR = JC_ERR_OK;


unordered_map<u32, string> Errs = {

  {JC_ERR_OK, "success"},
  {JC_ERR_InternalError, "internal error"},
  {JC_ERR_FileNOpened, "file not opened"},
  {JC_ERR_FileIsCorrupt, "file is corrupt"},
  {JC_ERR_InvalidIdentifier, "invalid identifier"},
  {JC_ERR_UnexpectedWord, "unexpected word"},
  {JC_ERR_InvalidValue, "invalid value"},
  {JC_ERR_ConvertToInteger, "convert to integer"},
  {JC_ERR_NestedOverflow, "nested overflow"},
};



extern "C" u32 jc_err_get()
{
  auto Temp = JC_ERROR;
  JC_ERROR = JC_ERR_OK;
  return Temp;
}

extern "C" const char* jc_err_str()
{
  auto Temp = JC_ERROR;
  JC_ERROR = JC_ERR_OK;
  return Errs[Temp].c_str();
}



#define Obj ((jc<cObj>*)__Obj)


// Is
extern "C" bool jc_is_val(jc_obj __Obj)
{
  return (__Obj != Nil) && ((*Obj)->Type == _c_Val);
}

extern "C" bool jc_is_int(jc_obj __Obj)
{
  return (__Obj != Nil) && ((*Obj)->Type == _c_Int);
}

extern "C" bool jc_is_bool(jc_obj __Obj)
{
  return (__Obj != Nil) && ((*Obj)->Type == _c_Bool);
}

extern "C" bool jc_is_float(jc_obj __Obj)
{
  return (__Obj != Nil) && ((*Obj)->Type == _c_Float);
}

extern "C" bool jc_is_data(jc_obj __Obj)
{
  return (__Obj != Nil) && ((*Obj)->Type == _c_Data);
}

extern "C" bool jc_is_stc(jc_obj __Obj)
{
  return (__Obj != Nil) && ((*Obj)->Type == _c_Stc);
}

extern "C" bool jc_is_arr(jc_obj __Obj)
{
  return (__Obj != Nil) && ((*Obj)->Type == _c_Arr);
}




// New / Dis
extern "C" jc_val jc_new_val(char *value)
{
  string Cac;
  if (value != Nil)
    Cac = string(value);


  return new jc<cObj>(
    new cVal(Cac)
  );
}

extern "C" jc_bool jc_new_int(u64 value)
{
  return new jc<cObj>(
    new cInt(value)
  );
}

extern "C" jc_bool jc_new_bool(bool value)
{
  return new jc<cObj>(
    new cBool(value)
  );
}

extern "C" jc_float jc_new_float(f64 value)
{
  return new jc<cObj>(
    new cFloat(value)
  );
}

extern "C" jc_data jc_new_data(data_64 value)
{
  return new jc<cObj>(
    new cData(value)
  );
}

extern "C" jc_stc jc_new_stc()
{
  return new jc<cObj>(
    new cStc()
  );
}

extern "C" jc_arr jc_new_arr(jc_obj value[], u32 count)
{
  vector<jc<cObj>> Cac;
  Cac.reserve(count);

  for (u32 i = 0; i < count; i++)
    Cac.push_back( *((jc<cObj>*)value[i]) );


  return new jc<cObj>(
    new cArr(Cac)
  );
}



extern "C" void jc_dis_obj(jc_obj __Obj)
{
  delete Obj;
}

extern "C" void jc_dis_str(char *Str)
{
  free(Str);
}



// Ref / Copy
jc<cObj> __Copy_Deep(jc<cObj> Source)
{
  switch (Source->Type)
  {
    // --- Basit Türler ---
    case _c_Val: {
      cVal* Org = (cVal*)Source.get();
      return jc<cObj>(new cVal(Org->Val));
    }

    case _c_Int: {
      cInt* Org = (cInt*)Source.get();
      return jc<cObj>(new cInt(Org->Val));
    }

    case _c_Bool: {
      cBool* Org = (cBool*)Source.get();
      return jc<cObj>(new cBool(Org->Val));
    }

    // --- Bellek Yönetimli Tür (Kritik) ---
    case _c_Data: {
      cData* Org = (cData*)Source.get();
      data_64 NewData;

      NewData.Size = Org->Val.Size;

      if (NewData.Size > 0 && Org->Val.Point != Nil)
      {
        // Yeni bellek alanı tahsis et
        NewData.Point = malloc(NewData.Size);
        // İçeriği kopyala
        memcpy(NewData.Point, Org->Val.Point, NewData.Size);
      }

      el {
        NewData.Point = Nil;
      }

      return jc<cObj>(new cData(NewData));
    }

    // --- Taşıyıcı Türler (Rekürsif) ---
    case _c_Arr: {
      cArr* Org = (cArr*)Source.get();
      vector<jc<cObj>> NewVec;

      NewVec.reserve(Org->Arr.size());

      for (auto &Item: Org->Arr)
        // Kendini tekrar çağır (Recursion)
        NewVec.push_back(__Copy_Deep(Item));

      return jc<cObj>(new cArr(NewVec));
    }

    case _c_Stc: {
      cStc* Org = (cStc*)Source.get();
      cStc* NewStc = new cStc(); // Yeni boş struct

      // Vektör kapasitesini ayarla
      NewStc->Stc.reserve(Org->Stc.size());

      for (auto &[Key, Val]: Org->Stc)
        // Anahtarı ve Değerin kopyasını ekle
        NewStc->Stc.push_back({ Key, __Copy_Deep(Val) });

      return jc<cObj>(NewStc);
    }
  }

  return Nil;
}

extern "C" jc_obj jc_copy(jc_obj __Obj)
{
  // Mevcut nesneye eriş (Macro: Obj -> ((jc<cObj>*)__Obj))
  jc<cObj> SourcePtr = *Obj;

  // Derin kopya oluştur
  jc<cObj> CopiedPtr = __Copy_Deep(SourcePtr);

  // C API için heap üzerinde yeni bir wrapper pointer döndür
  return new jc<cObj>(CopiedPtr);
}

extern "C" jc_obj jc_nref(jc_obj __Obj)
{
  return new jc<cObj>(*Obj);
}




#define SVal   ((cVal*)Obj->get())
#define SInt   ((cInt*)Obj->get())
#define SBool  ((cBool*)Obj->get())
#define SFloat ((cFloat*)Obj->get())
#define SData  ((cData*)Obj->get())
#define SStc   ((cStc*)Obj->get())
#define SArr   ((cArr*)Obj->get())


// Set/Get
extern "C" const char* jc_val_get(jc_val __Obj)
{
  return SVal->Val.c_str();
}

extern "C" void jc_val_set(jc_val __Obj, const char* Value)
{
  SVal->Val = string(Value);
}


extern "C" i64 jc_int_get(jc_int __Obj)
{
  return SInt->Val;
}

extern "C" void jc_int_set(jc_int __Obj, i64 Value)
{
  SInt->Val = Value;
}


extern "C" bool jc_bool_get(jc_bool __Obj)
{
  return SBool->Val;
}

extern "C" void jc_bool_set(jc_bool __Obj, bool Value)
{
  SBool->Val = Value;
}


extern "C" f64 jc_float_get(jc_float __Obj)
{
  return SFloat->Val;
}

extern "C" void jc_float_set(jc_float __Obj, f64 Value)
{
  SFloat->Val = Value;
}


extern "C" data_64 jc_data_get(jc_data __Obj)
{
  return SData->Val;
}

extern "C" void jc_data_set(jc_data __Obj, data_64 Value)
{
  SData->Val = Value;
}



extern "C" jc_obj jc_stc_get(jc_stc __Obj, const char* Name)
{
  for (auto &[Key, Val]: SStc->Stc)
    if (Key == string(Name))
      return new jc<cObj>(Val);

  return Nil;
}

extern "C" void jc_stc_set(jc_stc __Obj, const char* Name, jc_obj Object)
{
  for (auto &[Key, Val]: SStc->Stc)
    if (Key == string(Name))
    {
      Val = *(jc<cObj>*)Object;
      return;
    }

  SStc->Stc.push_back({
    string(Name), *(jc<cObj>*)Object
  });
}

extern "C" void jc_stc_del(jc_stc __Obj, const char* Name)
{
  SStc->Stc.erase(
    std::remove_if(SStc->Stc.begin(), SStc->Stc.end(),
      [&](const auto &X) {
        return X.first == string(Name);
      }
    ),
    SStc->Stc.end()
  );
}

extern "C" u32 jc_stc_count(jc_stc __Obj)
{
  return SStc->Stc.size();
}

extern "C" jc_obj jc_stc_index(jc_stc __Obj, u32 Index, char** Name)
{
  if (Name != Nil)
    *Name = strdup(SStc->Stc[Index].first.c_str());

  return new jc<cObj>(SStc->Stc[Index].second);
}

extern "C" void jc_stc_clear(jc_stc __Obj)
{
  SStc->Stc.clear();
}



extern "C" jc_obj jc_arr_get(jc_arr __Obj, u32 Index)
{
  return new jc<cObj>(SArr->Arr[Index]);
}

extern "C" void jc_arr_set(jc_arr __Obj, u32 Index, jc_obj Object)
{
  SArr->Arr[Index] = *(jc<cObj>*)Object;
}

extern "C" u32 jc_arr_count(jc_arr __Obj)
{
  return SArr->Arr.size();
}

extern "C" void jc_arr_push(jc_arr __Obj, jc_obj Object)
{
  SArr->Arr.push_back(*(jc<cObj>*)Object);
}

extern "C" void jc_arr_pushl(jc_arr __Obj, jc_obj Objects[], u32 Count)
{
  SArr->Arr.reserve(Count);

  for (u32 i = 0; i < Count; i++)
    SArr->Arr.push_back(*(jc<cObj>*)Objects[i]);
}

extern "C" void jc_arr_del(jc_arr __Obj, u32 Index)
{
  SArr->Arr[Index] = SArr->Arr.back();
  SArr->Arr.pop_back();
}

extern "C" void jc_arr_clear(jc_arr __Obj)
{
  SArr->Arr.clear();
}

#undef Obj
#undef SVal
#undef SInt
#undef SBool
#undef SFloat
#undef SData
#undef SStc
#undef SArr



// Parse
namespace
{
  string Parsers[] = {
    " ", "@", ".",":", ",",";",
    "(",")", "[","]", "{","}", "<",">"
  };

  enum eToken {
    Str,
    Int,
    Sym,
    Txt,
  };
}


string unquote(const string &Str)
{
  // En az 2 karakter olmalı ("")
  if (Str.size() < 2 || Str.front() != '"' || Str.back() != '"')
    return Str;

  string Raw = Str.substr(1, Str.size() -2);
  string Res;
  Res.reserve(Raw.size());

  for (u0 i = 0; i < Raw.size(); ++i)
  {
    // Kaçış karakteri mi? (\)
    if (Raw[i] == '\\' && i +1 < Raw.size())
    {
      switch (Raw[i +1])
      {
        case '"':  Res += '"';  break;
        case '\'': Res += '\''; break;
        case '\\': Res += '\\'; break;
        case 'a':  Res += '\a'; break;
        case 'b':  Res += '\b'; break;
        case 'e':  Res += '\e'; break;
        case 'f':  Res += '\f'; break;
        case 'n':  Res += '\n'; break;
        case 'r':  Res += '\r'; break;
        case 't':  Res += '\t'; break;
        case 'v':  Res += '\v'; break;

        // Bilinmeyen escape ise olduğu gibi bırak (örn: \a)
        default:
          Res += '\\';
          Res += Raw[i +1];
          break;
      }
      i++; // Bir sonraki karakteri (örn: n) atla, çünkü işledik.
    }
    else
      Res += Raw[i];
  }

  return Res;
}

inline eToken GetType(string Key)
{
  if (Key[0] == '"')
    return eToken::Str;

  if (isdigit(Key[0]))
    return eToken::Int;

  for (const auto &Delim: Parsers)
    if (Key.compare(0, Delim.size(), Delim) == 0)
      return eToken::Sym;


  return eToken::Txt;
}



// Parse
extern "C" jc_stc jc_parse(const char* FPath)
{
  bool Bin;

  _l_PreRead: {
    ifstream File(FPath, ios::binary);
    if (!File.is_open())
      return Nil;
      //throw runtime_error("File not opened: " +string(FPath));


    // Magic
    char Buf[10+7];
    File.read(&Buf[0], 10+7);

    Bin =
      (memcmp(&Buf[00], _c_Mag_Jixoid, 10) == 0) &&
      (memcmp(&Buf[10], _c_Mag_JConf,   7) == 0);
  }


  // Route
  return Bin ? jc_parse_bin(FPath):jc_parse_raw(FPath);
}


// ParseRaw
extern "C" jc_stc jc_parse_raw(const char* FPath)
{
  vector<string> Tokens;

  #pragma region Parse File

  ifstream File(FPath);
  if (!File.is_open())
  {
    JC_ERROR = JC_ERR_FileNOpened;
    return Nil;
  }


  string Temp;
  bool InString = false;


  string Line;
  while (getline(File, Line))
  {
    if (!Temp.empty() && !InString)
    {
      Tokens.push_back(Temp); // Önceki kelimeyi ekle
      Temp.clear();
    }

    if (InString)
      Temp += "\n";

    // Line Scan
    for (u0 x = 0; x < Line.size(); ++x)
    {
      // String
      if (Line[x] == '"')
      {
        // Geriye doğru kaçış karakterlerini say
        // Örn: "A\"" -> 1 slash (Escape var, string bitmez)
        // Örn: "A\\" -> 2 slash (Escape yok, slash escape edilmiş, string biter)
        int Backslashes = 0;
        int Check = x -1;
        while (Check >= 0 && Line[Check] == '\\') {
          Backslashes++;
          Check--;
        }

        // Eğer tek sayıda slash varsa, bu tırnak escape edilmiştir.
        // String durumunu değiştirme (InString aynı kalsın).
        bool IsEscaped = (Backslashes % 2 == 1);

        if (!IsEscaped)
          InString = !InString;
      }

      if (InString)
      {
        Temp += Line[x];
        continue;
      }




      // Space
      if (Line[x] == ' ')
      {
        if (! Temp.empty())
        {
          Tokens.push_back(Temp); // Önceki kelimeyi ekle
          Temp.clear();
        }
        continue;
      }

      // Comment
      if (Line[x] == '#')
        goto _l_NextLine;


      // Others
      for (const auto &Delim: Parsers)
        if (Line.compare(x, Delim.size(), Delim) == 0)
        {
          if (! Temp.empty())
          {

            Tokens.push_back(Temp); // Önceki kelimeyi ekle
            Temp.clear();
          }
          Tokens.push_back(Delim); // Delimiter'ı da ekle
          x += Delim.size() - 1; // Atlama yap

          goto _l_NextChar;
        }


      // Add
      Temp += Line[x];

      // Next
      _l_NextChar: {}
    }

    _l_NextLine: {}
  }

  if (!Temp.empty())
  {
    Tokens.push_back(Temp); // Önceki kelimeyi ekle
    Temp.clear();
  }


  File.close();
  #pragma endregion


  // Root
  jc<cObj> Root = jc<cObj>(new cStc());

  #pragma region Interpret

  #define isEnd  (Step >= Size)
  #define Next()  Step++; if (isEnd) goto _l_Escape;
  #define Word  Tokens[Step]

  u0 Size = Tokens.size();
  u0 Step = 0;

  // Root
  jc<cObj> Cov = Root;
  vector<jc<cObj>> Chain = {Root};


  // Start
  while (!isEnd)
  {
    string Name;

    _l_Start:

    // Exit Array
    if (Cov->Type == _c_Arr && Word[0] == ']')
    {
      // Change root
      Chain.pop_back();
      Cov = Chain.back();

      Next();

      goto _l_Start;
    }


    if (Cov->Type == _c_Arr)
    {
      Name = "";
      goto _l_Content;
    }


    // Exit Struct
    if (Cov->Type == _c_Stc && Word[0] == '}')
    {
      // Change root
      Chain.pop_back();
      Cov = Chain.back();

      Next();

      goto _l_Start;
    }



    // Name
    if (GetType(Word) != eToken::Txt)
    {
      JC_ERROR = JC_ERR_InvalidIdentifier;
      return Nil;
    }

    Name = Word;

    Next();

    // :
    if (Word != ":")
    {
      JC_ERROR = JC_ERR_UnexpectedWord;
      return Nil;
    }


    Next();


    _l_Content:


    // Bool
    if (Word == "true" || Word == "false" || Word == "yes" || Word == "no")
    {
      jc<cBool> Bool = jc<cBool>(new cBool());

      Bool->Val = (Word == "true" || Word == "yes");


      if (Cov->Type == _c_Arr)
        ((cArr*)Cov.get())->Arr.push_back(Bool);

      ef (Cov->Type == _c_Stc)
        ((cStc*)Cov.get())->Stc.push_back({Name, Bool});

      Next();
    }

    // String
    ef (GetType(Word) == eToken::Str)
    {
      jc<cVal> Val = jc<cVal>(new cVal());

      Val->Val = unquote(Word);


      if (Cov->Type == _c_Arr)
        ((cArr*)Cov.get())->Arr.push_back(Val);

      ef (Cov->Type == _c_Stc)
        ((cStc*)Cov.get())->Stc.push_back({Name, Val});

      Next();
    }

    // Integer / Float
    ef (GetType(Word) == eToken::Int || Word == "-")
    {
      string Temp;
      if (Word == "-")
      {
        Next();
        Temp = "-" +Word;
      }
      el
        Temp = Word;
      Next();


      bool IsFloat{};
      if (!isEnd && Word == ".")
      {
        IsFloat = true;
        Next();
        Temp += "." +Word;
        Next();
      }


      shared_ptr<cObj> O;

      if (IsFloat)
      {
        jc<cFloat> Flo = jc<cFloat>(new cFloat());
        auto eret = from_chars(Temp.data(), Temp.data() +Temp.size(), Flo->Val);
        if (eret.ec != std::errc())
        {
          JC_ERROR = JC_ERR_ConvertToInteger;
          return Nil;
        }
        O = Flo;
        
      }
      el
      {
        jc<cInt> Int = jc<cInt>(new cInt());
        auto eret = from_chars(Temp.data(), Temp.data() +Temp.size(), Int->Val);
        if (eret.ec != std::errc())
        {
          JC_ERROR = JC_ERR_ConvertToInteger;
          return Nil;
        }
        O = Int;
      }


      if (Cov->Type == _c_Arr)
        ((cArr*)Cov.get())->Arr.push_back(O);

      ef (Cov->Type == _c_Stc)
        ((cStc*)Cov.get())->Stc.push_back({Name, O});
    }

    // Data
    ef (Word[0] == '@')
    {
      jc<cData> Data = jc<cData>(new cData());
      Next();

      if (GetType(Word) != eToken::Str)
      {
        JC_ERROR = JC_ERR_UnexpectedWord;
        return Nil;
      }

      string DataPath = unquote(Word);
      ifstream BinFile(DataPath, ios::binary | ios::ate);

      if (!BinFile.is_open())
      {
        JC_ERROR = JC_ERR_FileNOpened;
        return Nil;
      }

      streamsize FSize = BinFile.tellg();
      BinFile.seekg(0, ios::beg);

      void *Buffer = malloc(FSize);
      if (Buffer == Nil)
      {
        JC_ERROR = JC_ERR_InternalError;
        return Nil;
      }

      BinFile.read((char*)Buffer, FSize);
      BinFile.close();

      Data->Val = {Buffer, (u0)FSize};


      if (Cov->Type == _c_Arr)
        ((cArr*)Cov.get())->Arr.push_back(Data);

      ef (Cov->Type == _c_Stc)
      ((cStc*)Cov.get())->Stc.push_back({Name, Data});

      Next();
    }

    // Raw Data
    ef (Word == "%BLOB")
    {
      jc<cData> Data = jc<cData>(new cData());

      Next();
      if (Word != ":")
      {
        JC_ERROR = JC_ERR_UnexpectedWord;
        return Nil;
      }


      Next();
      u64 DSize = 0;
      auto eret = from_chars(Word.data(), Word.data() + Word.size(), DSize);
      if (eret.ec != std::errc())
      {
        JC_ERROR = JC_ERR_ConvertToInteger;
        return Nil;
      }


      Next();
      if (Word != ":")
      {
        JC_ERROR = JC_ERR_UnexpectedWord;
        return Nil;
      }


      Next();
      if (DSize > 0)
      {
        void* Buffer = malloc(DSize);
        Data->Val = {Buffer, DSize};

        for (u0 i = 0; i < DSize; i++)
        {
          auto eret = from_chars(Word.data(), Word.data() + Word.size(), ((u8*)Buffer)[i], 16);
          if (eret.ec != std::errc())
          {
            JC_ERROR = JC_ERR_ConvertToInteger;
            return Nil;
          }

          Next();

          if (DSize > i)
            Next();
        }
      }
      else {
        Data->Val = {Nil, 0};
      }


      // Yapıya Ekle
      if (Cov->Type == _c_Arr)
        ((cArr*)Cov.get())->Arr.push_back(Data);

      ef (Cov->Type == _c_Stc)
        ((cStc*)Cov.get())->Stc.push_back({Name, Data});

      //Next();
    }

    // Struct
    ef (Word[0] == '{')
    {
      jc<cStc> Stc = jc<cStc>(new cStc());


      if (Cov->Type == _c_Arr)
        ((cArr*)Cov.get())->Arr.push_back(Stc);

      ef (Cov->Type == _c_Stc)
        ((cStc*)Cov.get())->Stc.push_back({Name, Stc});


      // Change root
      if (Chain.size() >= 256)
      {
        JC_ERROR = JC_ERR_NestedOverflow;
        return Nil;
      }
      Cov = Stc;
      Chain.push_back(Cov);

      Next();
    }

    // Array
    ef (Word[0] == '[')
    {
      jc<cArr> Arr = jc<cArr>(new cArr());

      if (Cov->Type == _c_Arr)
        ((cArr*)Cov.get())->Arr.push_back(Arr);

      ef (Cov->Type == _c_Stc)
        ((cStc*)Cov.get())->Stc.push_back({Name, Arr});


      // Change root
      if (Chain.size() >= 256)
      {
        JC_ERROR = JC_ERR_NestedOverflow;
        return Nil;
      }
      Cov = Arr;
      Chain.push_back(Cov);

      Next();
    }

    el {
      JC_ERROR = JC_ERR_InvalidValue;
      return Nil;
    }


    // ,
    if (Word[0] == ',')
      Next();
  }

  _l_Escape:


  #undef Word
  #undef Next
  #undef isEnd

  #pragma endregion


  jc<cObj> *Ret = new jc<cObj>(Root);

  return Ret;
}



// ParseBin
  cStc* __ParseBin_Stc(ifstream &Stream);

  cVal* __ParseBin_Val(ifstream &Stream)
  {
    // Ret
    cVal *Ret = new cVal();


    // Load Value
    u32 Tmp;

    Stream.read((c8*)(&Tmp), sizeof(u32));
    Ret->Val.resize(Tmp);
    Stream.read(&Ret->Val[0], Tmp);

    // Ret
    return Ret;
  }

  cInt* __ParseBin_Int(ifstream &Stream)
  {
    // Ret
    cInt *Ret = new cInt();


    // Load Value
    Stream.read((c8*)(&Ret->Val), sizeof(i64));

    // Ret
    return Ret;
  }

  cBool* __ParseBin_Bool(ifstream &Stream)
  {
    // Ret
    cBool *Ret = new cBool();


    // Load Value
    bool Tmp;
    Stream.read((c8*)(&Tmp), sizeof(bool));
    Ret->Val = Tmp;

    // Ret
    return Ret;
  }

  cFloat* __ParseBin_Float(ifstream &Stream)
  {
    // Ret
    auto Ret = new cFloat();


    // Load Value
    f64 Tmp;
    Stream.read((c8*)(&Tmp), sizeof(f64));
    Ret->Val = Tmp;

    // Ret
    return Ret;
  }

  cData* __ParseBin_Data(ifstream &Stream)
  {
    // Ret
    cData *Ret = new cData();


    // Load Value
    Stream.read((c8*)(&Ret->Val.Size), sizeof(Ret->Val.Size));
    Ret->Val.Point = malloc(Ret->Val.Size);

    Stream.read((c8*)(Ret->Val.Point), Ret->Val.Size);

    // Ret
    return Ret;
  }

  cArr* __ParseBin_Arr(ifstream &Stream)
  {
    // Ret
    cArr *Ret = new cArr();

    // Size
    i16 Child;
    Stream.read((c8*)(&Child), sizeof(i16));


    // Read Sub
    for (int i = 0; i < Child; i++)
    {

      // Read Magic
      i16 Tmp;
      Stream.read((c8*)(&Tmp), sizeof(i8));


      // Select Reader
      switch (Tmp)
      {
        case _c_Val:
          Ret->Arr.push_back(jc<cObj>(__ParseBin_Val(Stream)));
          break;

        case _c_Int:
          Ret->Arr.push_back(jc<cObj>(__ParseBin_Int(Stream)));
          break;

        case _c_Bool:
          Ret->Arr.push_back(jc<cObj>(__ParseBin_Bool(Stream)));
          break;

        case _c_Float:
          Ret->Arr.push_back(jc<cObj>(__ParseBin_Float(Stream)));
          break;

        case _c_Data:
          Ret->Arr.push_back(jc<cObj>(__ParseBin_Data(Stream)));
          break;

        case _c_Arr:
        {
          auto Temp = __ParseBin_Arr(Stream);
          if (Temp == Nil)
            return Nil;

          Ret->Arr.push_back(jc<cObj>(Temp));
          break;
        }

        case _c_Stc:
        {
          auto Temp = __ParseBin_Stc(Stream);
          if (Temp == Nil)
            return Nil;

          Ret->Arr.push_back(jc<cObj>(Temp));
          break;
        }


        default:
          JC_ERROR = JC_ERR_InternalError;
          return Nil;
      }
    }

    // Ret
    return Ret;
  }

  cStc* __ParseBin_Stc(ifstream &Stream)
  {
    // Ret
    cStc *Ret = new cStc();

    // Size
    i16 Child;
    Stream.read((c8*)(&Child), sizeof(i16));


    // Read Sub
    for (int i = 0; i < Child; i++)
    {
      // Name
      i16 Tmp;
      string Key;

      Stream.read((c8*)(&Tmp), sizeof(i16));
      Key.resize(Tmp);
      Stream.read(&Key[0], Tmp);


      // Read Magic
      Stream.read((c8*)(&Tmp), sizeof(i8));

      // Select Reader
      switch (Tmp)
      {
        case _c_Val:
          Ret->Stc.push_back({Key, jc<cObj>(__ParseBin_Val(Stream))});
          break;

        case _c_Int:
          Ret->Stc.push_back({Key, jc<cObj>(__ParseBin_Int(Stream))});
          break;

        case _c_Bool:
          Ret->Stc.push_back({Key, jc<cObj>(__ParseBin_Bool(Stream))});
          break;

        case _c_Float:
          Ret->Stc.push_back({Key, jc<cObj>(__ParseBin_Float(Stream))});
          break;

        case _c_Data:
          Ret->Stc.push_back({Key, jc<cObj>(__ParseBin_Data(Stream))});
          break;

        case _c_Arr:
        {
          auto Temp = __ParseBin_Arr(Stream);
          if (Temp == Nil)
            return Nil;

          Ret->Stc.push_back({Key, jc<cObj>(Temp)});
          break;
        }

        case _c_Stc:
        {
          auto Temp = __ParseBin_Stc(Stream);
          if (Temp == Nil)
            return Nil;

          Ret->Stc.push_back({Key, jc<cObj>(Temp)});
          break;
        }


        default:
          JC_ERROR = JC_ERR_InternalError;
          return Nil;
      }
    }

    // Ret
    return Ret;
  }


extern "C" jc_stc jc_parse_bin(const char* FPath)
{
  ifstream File(FPath, ios::binary);
  if (!File.is_open())
  {
    JC_ERROR = JC_ERR_FileNOpened;
    return Nil;
  }

  // Magic
  c8 Buf[10+7];
  File.read(&Buf[0], 10+7);


  if (memcmp(&Buf[00], _c_Mag_Jixoid, 10) != 0)
  {
    JC_ERROR = JC_ERR_FileIsCorrupt;
    return Nil;
  }

  if (memcmp(&Buf[10], _c_Mag_JConf, 7) != 0)
  {
    JC_ERROR = JC_ERR_FileIsCorrupt;
    return Nil;
  }



  // Read
  cStc *Ret = __ParseBin_Stc(File);

  File.close();


  // Ret
  return new jc<cStc>(Ret);
}




// WriteRaw
  bool __WriteRaw_Stc(ofstream &Stream, cStc *Content, string Prefix);

  void __WriteRaw_Val(ofstream &Stream, cVal *Content)
  {
    Stream << "\"" << Content->Val << "\"" << "\n";
  }

  void __WriteRaw_Int(ofstream &Stream, cInt *Content)
  {
    Stream << Content->Val << "\n";
  }

  void __WriteRaw_Bool(ofstream &Stream, cBool *Content)
  {
    Stream << (Content->Val ? "yes":"no") << "\n";
  }

  void __WriteRaw_Float(ofstream &Stream, cFloat *Content)
  {
    Stream << std::fixed << Content->Val << "\n";
  }

  void __WriteRaw_Data(ofstream &Stream, cData *Content)
  {
    Stream << "%BLOB" << ":" << Content->Val.Size << ":";

    Stream << hex;

    for (u0 i = 0; i < Content->Val.Size; i++)
      Stream << setw(2) << setfill('0') << (u16)((u8*)Content->Val.Point)[i] << ":";

    Stream << dec;

    Stream << "\n";
  }

  bool __WriteRaw_Arr(ofstream &Stream, cArr *Content, string Prefix = "") { for (auto &Objec: Content->Arr)
  {
    Stream << Prefix;

    switch (Objec->Type)
    {
      case _c_Val:
        __WriteRaw_Val(Stream, (cVal*)Objec.get());
        break;

      case _c_Int:
        __WriteRaw_Int(Stream, (cInt*)Objec.get());
        break;

      case _c_Bool:
        __WriteRaw_Bool(Stream, (cBool*)Objec.get());
        break;

      case _c_Float:
        __WriteRaw_Float(Stream, (cFloat*)Objec.get());
        break;

      case _c_Data:
        __WriteRaw_Data(Stream, (cData*)Objec.get());
        break;

      case _c_Arr:
        Stream << "[" << (((cArr*)Objec.get())->Arr.empty() ? "":"\n");
        if (!__WriteRaw_Arr(Stream, (cArr*)Objec.get(), Prefix +"  "))
          return false;
        Stream << Prefix << "]" "\n";
        break;

      case _c_Stc:
        Stream << "{" << (((cStc*)Objec.get())->Stc.empty() ? "":"\n");
        if (!__WriteRaw_Stc(Stream, (cStc*)Objec.get(), Prefix +"  "))
          return false;
        Stream << Prefix << "}" "\n";
        break;


      default:
        JC_ERROR = JC_ERR_InternalError;
        return false;
    }

  } return true; }

  bool __WriteRaw_Stc(ofstream &Stream, cStc *Content, string Prefix = "") { for (auto &[Key, Objec]: Content->Stc)
  {
    Stream << Prefix << Key << ": ";


    switch (Objec->Type)
    {
      case _c_Val:
        __WriteRaw_Val(Stream, (cVal*)Objec.get());
        break;

      case _c_Int:
        __WriteRaw_Int(Stream, (cInt*)Objec.get());
        break;

      case _c_Bool:
        __WriteRaw_Bool(Stream, (cBool*)Objec.get());
        break;

      case _c_Float:
        __WriteRaw_Float(Stream, (cFloat*)Objec.get());
        break;

      case _c_Data:
        __WriteRaw_Data(Stream, (cData*)Objec.get());
        break;

      case _c_Arr:
        Stream << "[" << (((cArr*)Objec.get())->Arr.empty() ? "":"\n");
        if (!__WriteRaw_Arr(Stream, (cArr*)Objec.get(), Prefix +"  "))
          return false;
        Stream << Prefix << "]" "\n";
        break;

      case _c_Stc:
        Stream << "{" << (((cStc*)Objec.get())->Stc.empty() ? "":"\n");
        if (!__WriteRaw_Stc(Stream, (cStc*)Objec.get(), Prefix +"  "))
          return false;
        Stream << Prefix << "}" "\n";
        break;


      default:
        JC_ERROR = JC_ERR_InternalError;
        return false;
    }

  } return true; }


extern "C" void jc_write_raw(const char* FPath, jc_stc Data)
{
  ofstream File(FPath);

  File << "# JixConf File" "\n\n";

  __WriteRaw_Stc(File, ((jc<cStc>*)Data)->get());

  File.close();
}



// WriteBin
  bool __WriteBin_Stc(ofstream &Stream, cStc *Content, bool NoMagic = false);

  void __WriteBin_Val(ofstream &Stream, cVal *Content)
  {
    // Magic
    Stream.write((c8*)(&_c_Val), sizeof(i8));


    // Save Value
    u32 Tmp = Content->Val.size();

    Stream.write((c8*)(&Tmp), sizeof(u32));
    Stream.write(&Content->Val[0], Tmp);
  }

  void __WriteBin_Int(ofstream &Stream, cInt *Content)
  {
    // Magic
    Stream.write((c8*)(&_c_Int), sizeof(i8));


    // Save Value
    Stream.write((c8*)(&Content->Val), sizeof(i64));
  }

  void __WriteBin_Bool(ofstream &Stream, cBool *Content)
  {
    // Magic
    Stream.write((c8*)(&_c_Bool), sizeof(i8));


    // Save Value
    Stream.write((c8*)(&Content->Val), sizeof(bool));
  }

  void __WriteBin_Float(ofstream &Stream, cFloat *Content)
  {
    // Magic
    Stream.write((c8*)(&_c_Float), sizeof(i8));


    // Save Value
    Stream.write((c8*)(&Content->Val), sizeof(f64));
  }

  void __WriteBin_Data(ofstream &Stream, cData *Content)
  {
    // Magic
    Stream.write((c8*)(&_c_Data), sizeof(i8));


    // Save Value
    Stream.write((c8*)(&Content->Val.Size), sizeof(u64));
    Stream.write((c8*)(Content->Val.Point), Content->Val.Size);
  }

  bool __WriteBin_Arr(ofstream &Stream, cArr *Content)
  {
    // Magic
    Stream.write((c8*)(&_c_Arr), sizeof(i8));

    // Size
    i16 Tmp = Content->Arr.size();

    Stream.write((c8*)(&Tmp), sizeof(i16));


    // Write Sub
    for (auto &Objec: Content->Arr) switch (Objec->Type)
    {
      case _c_Val:
        __WriteBin_Val(Stream, (cVal*)Objec.get());
        break;

      case _c_Int:
        __WriteBin_Int(Stream, (cInt*)Objec.get());
        break;

      case _c_Bool:
        __WriteBin_Bool(Stream, (cBool*)Objec.get());
        break;

      case _c_Float:
        __WriteBin_Float(Stream, (cFloat*)Objec.get());
        break;

      case _c_Data:
        __WriteBin_Data(Stream, (cData*)Objec.get());
        break;

      case _c_Arr:
        if (!__WriteBin_Arr(Stream, (cArr*)Objec.get()))
          return false;
        break;

      case _c_Stc:
        if (!__WriteBin_Stc(Stream, (cStc*)Objec.get()))
          return false;
        break;


      default:
        JC_ERROR = JC_ERR_InternalError;
        return false;
    }

    return true;
  }

  bool __WriteBin_Stc(ofstream &Stream, cStc *Content, bool NoMagic)
  {
    // Magic
    if (! NoMagic)
      Stream.write((c8*)(&_c_Stc), sizeof(i8));

    // Size
    i16 Tmp = Content->Stc.size();

    Stream.write((c8*)(&Tmp), sizeof(i16));


    // Write Sub
    for (auto &[Key, Objec]: Content->Stc)
    {
      // Name
      Tmp = Key.size();
      Stream.write((c8*)(&Tmp), sizeof(i16));
      Stream.write(&Key[0], Tmp);


      // Content
      switch (Objec->Type)
      {
        case _c_Val:
          __WriteBin_Val(Stream, (cVal*)Objec.get());
          break;

        case _c_Int:
          __WriteBin_Int(Stream, (cInt*)Objec.get());
          break;

        case _c_Bool:
          __WriteBin_Bool(Stream, (cBool*)Objec.get());
          break;

        case _c_Float:
          __WriteBin_Float(Stream, (cFloat*)Objec.get());
          break;

        case _c_Data:
          __WriteBin_Data(Stream, (cData*)Objec.get());
          break;

        case _c_Arr:
          if (!__WriteBin_Arr(Stream, (cArr*)Objec.get()))
            return false;
          break;

        case _c_Stc:
          if (!__WriteBin_Stc(Stream, (cStc*)Objec.get()))
            return false;
          break;

        default:
          JC_ERROR = JC_ERR_InternalError;
          return false;
      }
    }

    return true;
  }


extern "C" void jc_write_bin(const char* FPath, jc_stc Data)
{
  ofstream File(FPath, ios::binary);

  // Magic
  File.write(_c_Mag_Jixoid, 10);
  File.write(_c_Mag_JConf,  7);

  __WriteBin_Stc(File, ((jc<cStc>*)Data)->get(), true);

  File.close();
}

