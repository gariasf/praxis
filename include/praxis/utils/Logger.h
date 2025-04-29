#pragma once

#include <memory>
#include <string>
#include <spdlog/spdlog.h>

namespace spdlog
{
    class logger;
}

namespace praxis::utils
{

    /**
     * @class Logger
     * @brief Wrapper around spdlog providing logging capabilities
     */
    class Logger
    {
    public:
        /**
         * @brief Initialize the global logger
         * @param name Logger name
         * @param logFilePath Optional log file path, if empty will only log to console
         * @return True if initialization succeeded
         */
        static bool initialize(const std::string &name, const std::string &logFilePath = "");

        /**
         * @brief Shutdown the logger
         */
        static void shutdown();

        /**
         * @brief Log a trace message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void trace(spdlog::format_string_t<Args...> fmt, Args&&... args);

        /**
         * @brief Log a debug message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void debug(spdlog::format_string_t<Args...> fmt, Args&&... args);

        /**
         * @brief Log an info message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void info(spdlog::format_string_t<Args...> fmt, Args&&... args);

        /**
         * @brief Log a warning message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void warn(spdlog::format_string_t<Args...> fmt, Args&&... args);

        /**
         * @brief Log an error message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void error(spdlog::format_string_t<Args...> fmt, Args&&... args);

        /**
         * @brief Log a critical error message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void critical(spdlog::format_string_t<Args...> fmt, Args&&... args);

    private:
        static std::shared_ptr<spdlog::logger> s_logger;
    };

} // namespace praxis::utils

#include "Logger.inl"