#pragma once

#include <spdlog/spdlog.h>

namespace praxis::utils {

template <typename... Args>
void Logger::trace(const std::string& fmt, Args&&... args) {
  if (s_logger) {
    s_logger->trace(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::debug(const std::string& fmt, Args&&... args) {
  if (s_logger) {
    s_logger->debug(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::info(const std::string& fmt, Args&&... args) {
  if (s_logger) {
    s_logger->info(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::warn(const std::string& fmt, Args&&... args) {
  if (s_logger) {
    s_logger->warn(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::error(const std::string& fmt, Args&&... args) {
  if (s_logger) {
    s_logger->error(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::critical(const std::string& fmt, Args&&... args) {
  if (s_logger) {
    s_logger->critical(fmt, std::forward<Args>(args)...);
  }
}

} // namespace praxis::utils