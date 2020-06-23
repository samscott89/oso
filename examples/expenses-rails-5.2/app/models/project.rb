# frozen_string_literal: true

class Project < ApplicationRecord
  belongs_to :team
  belongs_to :location
  has_many :expenses
  has_one :organization, through: :team

  def total_spend
    expenses.pluck(:amount).sum
  end
end
