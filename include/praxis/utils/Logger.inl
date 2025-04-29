#pragma once

#include <spdlog/spdlog.h>

namespace praxis::utils {

template <typename... Args>
void Logger::trace(spdlog::format_string_t<Args...> fmt, Args&&... args) {
  if (s_logger) {
    s_logger->trace(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::debug(spdlog::format_string_t<Args...> fmt, Args&&... args) {
  if (s_logger) {
    s_logger->debug(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::info(spdlog::format_string_t<Args...> fmt, Args&&... args) {
  if (s_logger) {
    s_logger->info(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::warn(spdlog::format_string_t<Args...> fmt, Args&&... args) {
  if (s_logger) {
    s_logger->warn(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::error(spdlog::format_string_t<Args...> fmt, Args&&... args) {
  if (s_logger) {
    s_logger->error(fmt, std::forward<Args>(args)...);
  }
}

template <typename... Args>
void Logger::critical(spdlog::format_string_t<Args...> fmt, Args&&... args) {
  if (s_logger) {
    s_logger->critical(fmt, std::forward<Args>(args)...);
  }
}

} // namespace praxis::utils