import Loading from "../assets/loading.svg"
import { useState } from "preact/hooks"

export default function Component() {
    const [loading, setLoading] = useState(false)
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
                    <form className="space-y-6" action="/login" method="post">
                        <div>
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
                                />
                            </div>
                        </div>
                        <div>
                            <button
                                type="submit"
                                disabled={loading}
                                className="flex w-full justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:bg-indigo-500 dark:hover:bg-indigo-600 dark:focus:ring-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                { loading && <img src={Loading} className="w-5 h-5 mr-2 animate-spin" alt="loading" /> }
                                { loading ? "登录中..." : "连接 BUPT-portal" }
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </>


    )
}