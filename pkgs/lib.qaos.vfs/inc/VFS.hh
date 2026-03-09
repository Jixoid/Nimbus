/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include <cstddef>
#include <filesystem>
#include <istream>
#include <string_view>
#include <system_error>
#include <unordered_map>


using namespace std;


namespace vfs
{
	struct vfs_provider
	{
		void* (*Init)(){};
		void  (*Fini)(void*){};

		sptr<istream>  (*Open_RO)(void*, string_view FPath){};
		sptr<iostream> (*Open_RW)(void*, string_view FPath){};
	};


	extern unordered_map<string, pair<vfs_provider*, void*>> Providers;


	void Provider_Reg(string Protocol, vfs_provider*);
	void Provider_Del(string Protocol);



	inline sptr<istream>  Resolve_RO(string_view FPath)
	{
		string_view Payload;
		string Protocol;

		auto SplitPos = FPath.find("://", 1);


		if (SplitPos != std::string_view::npos) [[likely]]
		{
			Protocol = FPath.substr(0, SplitPos);
			Payload  = FPath.substr(SplitPos+3);
		}

		else [[unlikely]]
		{
			Protocol = "file";
			Payload  = FPath;
		}

		if (auto X = vfs::Providers.find(Protocol); X != vfs::Providers.end())
			return X->second.first->Open_RO(X->second.second, Payload);
		else
			throw filesystem::filesystem_error("File can not opened", FPath, std::make_error_code(std::errc::no_such_device_or_address));
	}

	inline sptr<iostream> Resolve_RW(string_view FPath)
	{
		string_view Payload;
		string Protocol;

		auto SplitPos = FPath.find("://", 1);


		if (SplitPos != std::string_view::npos) [[likely]]
		{
			Protocol = FPath.substr(0, SplitPos);
			Payload  = FPath.substr(SplitPos+3);
		}

		else [[unlikely]]
		{
			Protocol = "file";
			Payload  = FPath;
		}

		if (auto X = vfs::Providers.find(Protocol); X != vfs::Providers.end())
			return X->second.first->Open_RW(X->second.second, Payload);
		else
			throw filesystem::filesystem_error("File can not opened", FPath, std::make_error_code(std::errc::no_such_device_or_address));
	}

}




inline sptr<istream> operator""_vfs_ro (const char *_FPath, std::size_t _Len)
{
	return vfs::Resolve_RO(string_view(_FPath, _Len));
}


inline sptr<iostream> operator""_vfs_rw (const char *_FPath, std::size_t _Len)
{
	return vfs::Resolve_RW(string_view(_FPath, _Len));	
}

