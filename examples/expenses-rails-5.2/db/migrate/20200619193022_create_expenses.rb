# frozen_string_literal: true

class CreateExpenses < ActiveRecord::Migration[5.2]
  def change
    create_table :expenses do |t|
      t.references :user, foreign_key: true
      t.integer :amount
      t.string :description
      t.references :project, foreign_key: true

      t.timestamps
    end
  end
end
