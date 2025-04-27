#pragma once

#include <memory>
#include <string>

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
        static void trace(const std::string &fmt, Args &&...args);

        /**
         * @brief Log a debug message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void debug(const std::string &fmt, Args &&...args);

        /**
         * @brief Log an info message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void info(const std::string &fmt, Args &&...args);

        /**
         * @brief Log a warning message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void warn(const std::string &fmt, Args &&...args);

        /**
         * @brief Log an error message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void error(const std::string &fmt, Args &&...args);

        /**
         * @brief Log a critical error message
         * @param fmt Format string
         * @param args Format arguments
         */
        template <typename... Args>
        static void critical(const std::string &fmt, Args &&...args);

    private:
        static std::shared_ptr<spdlog::logger> s_logger;
    };

} // namespace praxis::utils

#include "Logger.inl"