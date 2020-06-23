# frozen_string_literal: true

class ApplicationController < ActionController::Base
  before_action :current_user
  before_action :oso

  private

  def current_user
    @current_user ||= session[:current_user_id] &&
                      User.find_by(id: session[:current_user_id])
  end

  def oso
    @oso ||= Rails.configuration.oso
  end

  def allowed(resource)
    oso.allow(actor: current_user, action: action_name, resource: resource)
  end
end
