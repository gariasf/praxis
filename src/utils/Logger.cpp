#include "praxis/utils/Logger.h"

#include <iostream>
#include <spdlog/sinks/basic_file_sink.h>
#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

namespace praxis::utils {

std::shared_ptr<spdlog::logger> Logger::s_logger;

bool Logger::initialize(const std::string& name, const std::string& logFilePath) {
  try {
    // Check if logger with this name already exists
    auto existing_logger = spdlog::get(name);
    if (existing_logger) {
      s_logger = existing_logger;
      spdlog::set_default_logger(s_logger);
      info("Logger reused");
      return true;
    }

    // Check if logger with this name already exists
    auto existing_logger = spdlog::get(name);
    if (existing_logger) {
      s_logger = existing_logger;
      spdlog::set_default_logger(s_logger);
      info("Logger reused");
      return true;
    }

    // Create sinks
    std::vector<spdlog::sink_ptr> sinks;

    // Console sink
    auto consoleSink = std::make_shared<spdlog::sinks::stdout_color_sink_mt>();
    consoleSink->set_pattern("[%^%l%$] %v");
    sinks.push_back(consoleSink);

    // File sink (optional)
    if (!logFilePath.empty()) {
      auto fileSink = std::make_shared<spdlog::sinks::basic_file_sink_mt>(logFilePath, true);
      fileSink->set_pattern("[%H:%M:%S.%e] [%l] %v");
      sinks.push_back(fileSink);
    }

    // Create and register logger
    s_logger = std::make_shared<spdlog::logger>(name, sinks.begin(), sinks.end());
    s_logger->set_level(spdlog::level::trace);
    s_logger->flush_on(spdlog::level::trace);

    spdlog::register_logger(s_logger);
    spdlog::set_default_logger(s_logger);

    info("Logger initialized");
    return true;
  } catch (const spdlog::spdlog_ex& ex) {
    std::cerr << "Logger initialization failed: " << ex.what() << std::endl;
    return false;
  }
}

void Logger::shutdown() {
  if (s_logger) {
    info("Logger shutting down");
    spdlog::shutdown();
    s_logger.reset();
  }
}

} // namespace praxis::utils