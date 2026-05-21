/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qcl/ds/ds.hh"
#include "qcl/widget.hh"
#include "qstd/monet.hh"

using namespace qcl;



namespace qstd
{

  enum grid_col_type 
  {
    gctText,
    gctCheckbox,
    gctButton,
  };

  struct grid_column
  {
    string Title;
    string FieldName;
    i32 Width = 100;
    
    grid_col_type Type = grid_col_type::gctText;
  };

  struct qcl_object grid: qcl::widget
  {
    private:
      vector<grid_column> m_cols;
      vector<vector<string>> m_rows;

      u32 m_rowHeight{24};
      u32 m_headerHeight{30};

      dcolor m_lineColor{this, "monet(SurfaceVariant)"};

    public:
      inline fun cols() const -> const vector<grid_column>& { return m_cols; }
      inline fun setCols(const vector<grid_column>& val) -> void { m_cols = val; update(visDirtyDraw); }

      inline fun rows() const -> const vector<vector<string>>& { return m_rows; }
      inline fun setRows(const vector<vector<string>>& val) -> void { m_rows = val; update(visDirtyDraw); }

      inline fun rowHeight() const noexcept -> u32 { return m_rowHeight; }
      inline fun setRowHeight(u32 val) -> void { m_rowHeight = val; update(visDirtyDraw); }
      qcl_prop(TYPE u32, NAME rowHeight, READ rowHeight, WRITE setRowHeight);
      
      inline fun headerHeight() const noexcept -> u32 { return m_headerHeight; }
      inline fun setHeaderHeight(u32 val) -> void { m_headerHeight = val; update(visDirtyDraw); }
      qcl_prop(TYPE u32, NAME headerHeight, READ headerHeight, WRITE setHeaderHeight);
      
      inline fun lineColor() -> dcolor& { return m_lineColor; }


    public:
      qcl_func qev<void, i32 /* Row */, i32 /* Col */> onCellClick;

    public:
      fun draw(renderContext&) -> void override;

      fun calcAutoSize(renderContext&) -> void override;

      fun doReset(renderContext&) -> void override;
      fun doClickEx(poit<f32> Pos) -> void override;
      virtual fun doCellClick(i32 Row, i32 Col) -> void;

    public:
      fun getProp(string_view) -> expected<std::any, bool> override;
      fun setProp(string_view, std::any) -> expected<void, bool> override;
      fun loadProp(string_view, const ds::value&, function<expected<qev_seed,bool>(string_view)>) -> expected<bool, string> override;
  };

}
