# frozen_string_literal: true

class CreateUsers < ActiveRecord::Migration[5.2]
  def change
    create_table :users do |t|
      t.string :email
      t.string :title
      t.belongs_to :location
      t.belongs_to :organization
      t.references :manager

      t.timestamps
    end
  end
end
