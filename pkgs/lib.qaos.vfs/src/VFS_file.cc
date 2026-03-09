/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "Basis.h"
#include "VFS.hh"

#include <fstream>
#include <iostream>
#include <istream>
#include <string>
#include <string_view>
#include <dlfcn.h>


namespace {


vfs::vfs_provider PROV = {
	
	.Init = []() -> void*
	{
		return Nil;
	},

	#define Data ((vfs_data*)_Data)

	.Fini = [](void* _Data)
	{},


	.Open_RO = [](void* _Data, string_view FPath) -> sptr<istream>
	{
		return sptr<istream>(new ifstream(string(FPath)));
	},

	.Open_RW = [](void* _Data, string_view FPath) -> sptr<iostream>
	{
		return sptr<iostream>(new fstream(string(FPath)));
	},

};



struct x
{
	x()  { vfs::Provider_Reg("file", &PROV); }
	~x() { vfs::Provider_Del("file"); }
	
}X;

}
