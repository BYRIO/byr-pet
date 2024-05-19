import Loading from "../assets/loading.svg"
import { useState, useRef } from "preact/hooks"

export default function Component() {
    const [loading, setLoading] = useState(false)
    const usernameRef = useRef(null)
    const passwordRef = useRef(null)
    const [errorMsg, setErrorMsg] = useState('')
    const [loggedIn, setLoggedIn] = useState(false)

    async function submit(e) {
        e.preventDefault()
        setLoading(true)
        try {
            const response = await fetch('/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                },
                body: `username=${encodeURIComponent(usernameRef.current.value)}&password=${encodeURIComponent(passwordRef.current.value)}`
            })
            const result = await response.json()
            if (result.code) {
                setErrorMsg(result.message)
            } else {
                setLoggedIn(true)
            }
        } catch (error) {
            console.error(error)
            setErrorMsg('连接失败: ' + error.message)
        } finally {
            setLoading(false)
        }
    }

    if (loggedIn) {
        return (
            <div className="min-h-[75vh] flex flex-col items-center justify-center px-4 py-12">
                <div className="w-full max-w-md space-y-8">
                    <div className="text-center">
                        <svg
                            className="mx-auto h-24 w-24 text-green-500"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            xmlns="http://www.w3.org/2000/svg"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M5 13l4 4L19 7"
                            ></path>
                        </svg>
                        <h1 className="text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-50">
                            登录成功
                        </h1>
                        <p className="mt-2 text-xs text-gray-600 dark:text-gray-400">
                            现在您可以断开 BYR-pet Wi-Fi 连接
                        </p>
                    </div>
                </div>
            </div>
        )
    }

    return (
        <>
            <div className="min-h-[75vh] flex flex-col items-center justify-center px-4 py-12">
                <div className="w-full max-w-md space-y-8">
                    <div className="text-center">
                        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-gray-50">
                            Welcome to BYR-pet
                        </h1>
                        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            输入学号和校园网密码以连接 BUPT-portal
                        </p>
                    </div>
                    <div className="space-y-6">
                        <div>
                            {errorMsg && (<div className="text-red-500 dark:text-red-400 text-center text-sm mb-4">
                                {errorMsg}
                            </div>)}
                            <label
                                htmlFor="username"
                                className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                            >
                                学号
                            </label>
                            <div className="mt-1">
                                <input
                                    id="username"
                                    autoComplete="username"
                                    required={true}
                                    className="block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 placeholder-gray-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-50 dark:placeholder-gray-500"
                                    type="text"
                                    name="username"
                                    ref={usernameRef}
                                />
                            </div>
                        </div>
                        <div>
                            <label
                                htmlFor="password"
                                className="block text-sm font-medium text-gray-700 dark:text-gray-300"
                            >
                                密码
                            </label>
                            <div className="mt-1">
                                <input
                                    id="password"
                                    autoComplete="current-password"
                                    required={true}
                                    className="block w-full appearance-none rounded-md border border-gray-300 px-3 py-2 placeholder-gray-400 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-50 dark:placeholder-gray-500"
                                    type="password"
                                    name="password"
                                    ref={passwordRef}
                                />
                            </div>
                        </div>
                        <div>
                            <button
                                type="submit"
                                disabled={loading}
                                onClick={submit}
                                className="flex w-full justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-600 dark:focus:ring-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {loading && <img src={Loading} className="w-5 h-5 mr-2 animate-spin" alt="loading" />}
                                {loading ? "登录中..." : errorMsg ? "重试" : "连接 BUPT-portal"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </>
    )
}