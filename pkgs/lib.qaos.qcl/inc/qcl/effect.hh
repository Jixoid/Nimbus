/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "ds/ds.hh"
#include <expected>
#include <string_view>
#include "include/core/SkPaint.h"

using namespace std;



namespace qcl
{

  /**
   * @brief Base class for visual effects.
   * 
   * Defines the interface for effects that can modify drawing properties.
   */
  struct effect
  {
    public:
      /**
       * @brief Default constructor.
       */
      inline effect() {}

      /**
       * @brief Virtual destructor.
       */
      inline virtual ~effect() {}

    private:
      bool m_enabled{true}; ///< Indicates whether the effect is active.

    public:
      /**
       * @brief Checks if the effect is enabled.
       * @return True if the effect is enabled, false otherwise.
       */
      inline fun enabled() const noexcept -> bool {return m_enabled;}

      /**
       * @brief Enables or disables the effect.
       * @param Val Boolean value indicating the new state.
       */
      inline fun setEnabled(bool Val) noexcept -> void {m_enabled = Val;}

    public:
      /**
       * @brief Resets the effect to its default state.
       */
      virtual fun reset() -> void {};

      /**
       * @brief Modifies the provided SkPaint object with the effect's properties.
       * @param Paint The SkPaint object to be modified.
       */
      virtual fun modify(SkPaint& Paint) -> void = 0;

      /**
       * @brief Loads a property for the effect dynamically.
       * @param Name The name of the property.
       * @param Prop The value of the property.
       * @return An expected boolean indicating success, or an error message string on failure.
       */
      virtual fun loadProp(string_view Name, const ds::value& Prop) -> std::expected<bool, std::string>;
  };

}
