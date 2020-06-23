# frozen_string_literal: true

class SessionsController < ApplicationController
  def create
    user = User.find_by(email: params[:email])
    if user
      session[:current_user_id] = user.id
      redirect_back fallback_location: root_path
    else
      redirect_back fallback_location: root_path, notice: 'Unregistered user.'
    end
  end

  def destroy
    session.delete(:current_user_id)
    @current_user = nil
    redirect_back fallback_location: root_path
  end
end
