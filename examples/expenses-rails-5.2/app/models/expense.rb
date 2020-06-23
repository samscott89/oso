# frozen_string_literal: true

class Expense < ApplicationRecord
  belongs_to :employee, class_name: 'User', foreign_key: 'user_id'
  belongs_to :project
end
