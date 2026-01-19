/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#define el else
#define ef else if

#include "Basis.h"

#include "qcl/Control.hh"
#include "qcl/Graphic.hh"
#include "qcl/Platform.hh"
#include "qcl/DynamicLoad.hh"
#include "qgl/QGL.hh"

using namespace std;
using namespace jix;
using namespace qcl;



namespace qgl
{

  #pragma region viewport

  viewport::viewport()
    : control()
  {
    qcl::platform::opengl::New(&GLContext, Size.W, Size.H);
  }

  viewport::~viewport()
  {
    qcl::platform::opengl::Dis(GLContext);
  }


  void viewport::SetCurrent()
  {
    qcl::platform::opengl::Set_Current(GLContext);
  }

  void viewport::Do_Resize()
  {
    SetCurrent();
    platform::opengl::Set_Size(GLContext, max(0,Size.W), max(0,Size.H));
  
    control::Do_Resize();
  }

  void viewport::Draw()
  {
    SetCurrent();
  }

  void viewport::Draw_after()
  {
    SetCurrent();

    u32 *Data = Surface->Get_Data();

    qcl::platform::opengl::Flush(GLContext);
    qcl::platform::opengl::ReadBuffer(GLContext, Data);

    Surface->Set_Dirty();
  }

  #pragma endregion



  void Register()
  {
    #define Reg(X)  dyn::Register(#X, []() -> control* { return new X(); })
    #define RegE(X) dyn::Register(#X, []() -> effect*  { return new X(); })

    Reg(qgl::viewport);

    #undef Reg
  }

}
