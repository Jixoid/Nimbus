/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <iostream>

#include "Basis.h"

#include "qcl/Control.hh"
#include "qcl/Application.hh"
#include "qcl/DynamicLoad.hh"
#include "qstd/Standard.hh"


using namespace std;
using namespace jix;
using namespace qcl;



#define DECLARE_SUB(Name, Type) Type *Name;
#define INIT_SUB(Name, Type) Name = dynamic_cast<Type*>(qcl::dyn::findFromName(this, #Name).get());
#define INIT_EV(Class, Name) {#Name, qev_seed(this, &Class::Name)},
#define SUBCOM_GENERATE(File, Class, Events, Comps) \
public: \
  Comps(DECLARE_SUB) \
  Class() { \
    qcl::dyn::loadFromFile(this, File, {Events(INIT_EV)}); \
    Comps(INIT_SUB) \
  }



class MainForm: public qstd::form
{
public:
  #define COMPONENTS(DO)


  #define EVENTS(DO) \
    DO(MainForm, ThemeChange)

  
  SUBCOM_GENERATE("Des/Main.qdl", MainForm, EVENTS, COMPONENTS)


public:
  void ThemeChange(control *Self, i32 TabID)
  {
    qstd::Monet.Update(TabID == 0);

    CurrentApp->PushMessage(this, controlMessages::cmReset);
  }

};



extern "C" int qcl_entry2(ohid App_H)
{
  application App(App_H, true);


  qstd::Register();


  qstd::Monet.Update(color(203.0/255.0, 151.0/255.0, 77.0/255.0), true);
    //.Update(color(49.0/255.0, 211.0/255.0, 76.0/255.0), true);
    //.Update(color(0.45, 0.30, 0.80), true);


  auto AForm = new MainForm();
  AForm->Show();

  CurrentApp->Run();
  return 0;
}

extern "C" void qcl_error(const char *Msg)
{
  cerr << Msg << endl;
}

