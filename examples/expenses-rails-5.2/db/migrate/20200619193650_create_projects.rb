# frozen_string_literal: true

class CreateProjects < ActiveRecord::Migration[5.2]
  def change
    create_table :projects do |t|
      t.belongs_to :team
      t.belongs_to :location
      t.string :name

      t.timestamps
    end
  end
end
