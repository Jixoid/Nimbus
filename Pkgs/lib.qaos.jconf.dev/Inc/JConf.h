/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "Basis.h"


#ifdef __cplusplus
extern "C" {
#endif


// Types
typedef  point  jc_obj;

typedef  jc_obj  jc_val;
typedef  jc_obj  jc_int;
typedef  jc_obj  jc_bool;
typedef  jc_obj  jc_float;
typedef  jc_obj  jc_data;
typedef  jc_obj  jc_stc;
typedef  jc_obj  jc_arr;



// Errors
const u32
  JC_ERR_OK = 0,
  JC_ERR_InternalError = 1,
  JC_ERR_FileNOpened = 2,
  JC_ERR_FileIsCorrupt = 3,
  JC_ERR_InvalidIdentifier = 4,
  JC_ERR_UnexpectedWord = 5,
  JC_ERR_InvalidValue = 6,
  JC_ERR_ConvertToInteger = 7,
  JC_ERR_NestedOverflow = 8;

u32 jc_err_get();
const char* jc_err_str();



// Is
bool jc_is_val(jc_obj);
bool jc_is_int(jc_obj);
bool jc_is_bool(jc_obj);
bool jc_is_float(jc_obj);
bool jc_is_data(jc_obj);
bool jc_is_stc(jc_obj);
bool jc_is_arr(jc_obj);



// New / Dis
jc_val   jc_new_val(char* value);
jc_val   jc_new_int(u64  value);
jc_bool  jc_new_bool(bool  value);
jc_float jc_new_float(f64  value);
jc_data  jc_new_data(data_64 value);
jc_stc   jc_new_stc();
jc_arr   jc_new_arr(jc_obj value[], u32 count);

void jc_dis_obj(jc_obj obj);
void jc_dis_str(char *str);



// Ref / Copy
jc_obj jc_copy(jc_obj);
jc_obj jc_nref(jc_obj);



// Set / Get
const char* jc_val_get(jc_val);
void jc_val_set(jc_val, const char* value);

i64  jc_int_get(jc_int);
void jc_int_set(jc_int, i64 value);

bool jc_bool_get(jc_bool);
void jc_bool_set(jc_bool, bool value);

f64  jc_float_get(jc_float);
void jc_float_set(jc_float, f64 value);

data_64 jc_data_get(jc_val);
void    jc_data_set(jc_val, data_64 value);

jc_obj jc_stc_get  (jc_stc, const char* scope);
void   jc_stc_set  (jc_stc, const char* scope, jc_obj object);
void   jc_stc_del  (jc_stc, const char* scope);
u32    jc_stc_count(jc_stc);
jc_obj jc_stc_index(jc_stc, u32 index, char** Name);
void   jc_stc_clear(jc_stc);

jc_obj jc_arr_get  (jc_arr, u32 index);
void   jc_arr_set  (jc_arr, u32 index, jc_obj object);
u32    jc_arr_count(jc_arr);
void   jc_arr_push (jc_arr, jc_obj object);
void   jc_arr_pushl(jc_arr, jc_obj value[], u32 count);
void   jc_arr_del  (jc_arr, u32 index);
void   jc_arr_clear(jc_arr);



// Parse
jc_stc jc_parse(const char* fpath);
jc_stc jc_parse_raw(const char* fpath);
jc_stc jc_parse_bin(const char* fpath);

void jc_write_raw(const char* fpath, jc_stc data);
void jc_write_bin(const char* fpath, jc_stc data);


#ifdef __cplusplus
}
#endif
