/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qcl/Animator.hh"
#include "qcl/Visual.hh"
#include <chrono>

using namespace std;


namespace qcl::anim
{
  vector<anim_node> Anims;

  
  static u64 AnimCounter{};
  fun GetAnimID() -> u64 {
    return (++AnimCounter == 0)
      ? ++AnimCounter
      : AnimCounter;
  };


  fun AnimPush(anim_node Anim) -> u64
  {
    using namespace chrono;

    Anim.ID = GetAnimID();
    Anim.BegT = (u64)duration_cast<milliseconds>(steady_clock::now().time_since_epoch()).count();
    Anims.push_back(Anim);

    Anim.Object->update();
    return Anim.ID;
  }

  fun AnimSwap(anim_node Anim, u64 AnimID) -> u64
  {
    using namespace chrono;
    Anim.BegT = (u64)duration_cast<milliseconds>(steady_clock::now().time_since_epoch()).count();

    if (AnimID)
      for (auto &A: Anims)
        if (A.ID == AnimID)
        {
          Anim.ID = AnimID;
          A = Anim;
          return AnimID;
        }
        
        
    Anim.ID = GetAnimID();
    Anims.push_back(Anim);
    
    Anim.Object->update();
    return Anim.ID;
  }
  

  fun AnimDel(u64 ID) -> bool
  {
    auto it = remove_if(Anims.begin(), Anims.end(), [&](anim_node& node) -> bool
    {
      return (node.ID == ID);
    });

    if (it != Anims.end())
      Anims.erase(it, Anims.end());

    return it != Anims.end();
  }




  fun AnimWorker() -> void
  {
    using namespace chrono;
    
    u64 Now = duration_cast<milliseconds>(steady_clock::now().time_since_epoch()).count();


    auto it = remove_if(Anims.begin(), Anims.end(), [&](anim_node& node) -> bool
    {
      if (Now < node.BegT +node.Interval) {

        if (node.Func) {
          f32 Time = f32(Now -node.BegT) / node.Interval;
          auto result = node.Func(node.Beg, node.End, Time);

          std::visit([&](auto funcPtr) {
            using ArgT = typename member_traits<decltype(funcPtr)>::arg_t;
            
            (node.Object->*funcPtr)(std::get<ArgT>(result));  
          }, node.Target);
        }
        else
          node.Object->update();
        
        return false;
      } else {
        if (node.Func)
          std::visit([&](auto funcPtr) {
            using ArgT = typename member_traits<decltype(funcPtr)>::arg_t;
            
            (node.Object->*funcPtr)(std::get<ArgT>(node.End));  
          }, node.Target);
        else
          node.Object->update();

        return true;
      }
    });

    if (it != Anims.end())
      Anims.erase(it, Anims.end());
  }

}
