/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/



#pragma once

#include "DS.hh"
#include "include/core/SkPaint.h"
#include "include/core/SkShader.h"
#include "include/core/SkColorSpace.h"
#include "qcl/Effect.hh"
#include "qcl/DynamicLoad.hh"

using namespace std;
using namespace qcl;



namespace qstd
{
  fun effs() -> dyn::effs;



  struct noise: effect
  {
    public:
      inline noise(): effect() { reset(); };
      inline ~noise() override {}


    protected:
      const u32 N_SIZE = 256;


    private:
      f32 m_vnoise = 0.13;
      sk_sp<SkShader> m_noiseShader;


    public:
      inline fun vnoise() const noexcept -> f32 { return m_vnoise; }
      inline fun setVNoise(f32 val) -> void { m_vnoise = val; reset(); }

    
    public:
      fun reset() -> void override;
      fun modify(SkPaint&) -> void override;

      fun loadProp(string_view Name, const ds::value& Prop) -> std::expected<bool, std::string> override;
  };


  struct grayscale: effect
  {
    public:
      inline grayscale(): effect() {};
      inline ~grayscale() override {}

    public:
      fun modify(SkPaint&) -> void override;
  };

}
