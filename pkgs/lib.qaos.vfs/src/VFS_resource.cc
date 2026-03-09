/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "Basis.h"
#include "VFS.hh"

#include <iostream>
#include <span>
#include <spanstream>
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


	.Open_RO = [](void* _Data, string_view nFPath) -> sptr<istream>
	{
		string FPath(nFPath);
		for (char &C: FPath)
			if (C == '.' | C == '/' | C == '-')
				C = '_';


		auto Sym_Beg = dlsym(RTLD_DEFAULT, ("_binary_res_"+FPath+"_start").c_str());
		auto Sym_End = dlsym(RTLD_DEFAULT, ("_binary_res_"+FPath+"_end").c_str());

		if (!Sym_Beg || !Sym_End)
			throw filesystem::filesystem_error("Embedded file not opened", nFPath, std::make_error_code(std::errc::no_such_file_or_directory));

		auto Ret = new ispanstream(span<char>((char*)Sym_Beg, (u0)((u0)Sym_End -(u0)Sym_Beg)));
		return sptr<istream>(Ret);
	},

	.Open_RW = [](void* _Data, string_view FPath) -> sptr<iostream>
	{
		throw filesystem::filesystem_error("Embedded file not opened with RW", FPath, std::make_error_code(std::errc::read_only_file_system));
	},

};



struct x
{
	x()  { vfs::Provider_Reg("resource", &PROV); }
	~x() { vfs::Provider_Del("resource"); }
	
}X;

}
