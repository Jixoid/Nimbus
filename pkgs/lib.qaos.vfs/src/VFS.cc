/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "VFS.hh"


using namespace std;


namespace vfs
{
	unordered_map<string, pair<vfs_provider*, void*>> Providers;


  void Provider_Reg(string Protocol, vfs_provider *Provider)
  {
    void* Data = Provider->Init();

    Providers[Protocol] = {Provider, Data};
  }

	void Provider_Del(string Protocol)
  {
    if (auto X = Providers.find(Protocol); X != Providers.end())
    {
      X->second.first->Fini(X->second.second);

      Providers.erase(X);
    }
  }
}

