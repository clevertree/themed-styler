export default function StyleKeywordsTest() {
    return <body className="w-full h-auto p-4 gap-4 bg-surface" test-id="style-test-body">
        <div className="bg-primary p-4 w-full rounded" test-id="header">
            <h1 className="text-white">Style Keywords Test</h1>
            <p className="text-white text-sm opacity-80">Verifying fit-content and auto mapping</p>
        </div>

        <div className="flex flex-col gap-4" test-id="test-cases">
            {/* Test Case 1: h-fit (fit-content) */}
            <div className="p-4 bg-white rounded border border-themed h-fit" test-id="h-fit-container">
                <span className="font-bold">h-fit container</span>
                <p className="text-sm text-muted">This container should wrap its content tightly.</p>
                <div className="mt-2 p-2 bg-blue-100 rounded">
                    Content
                </div>
            </div>

            {/* Test Case 2: h-auto (auto) */}
            <div className="p-4 bg-white rounded border border-themed h-auto" test-id="h-auto-container">
                <span className="font-bold">h-auto container</span>
                <p className="text-sm text-muted">This container should also wrap its content (auto -> wrap_content).</p>
                <div className="mt-2 p-2 bg-green-100 rounded">
                    More Content
                </div>
            </div>

            {/* Test Case 3: w-fit (fit-content) */}
            <div className="p-4 bg-white rounded border border-themed w-fit" test-id="w-fit-container">
                <span className="font-bold">w-fit</span>
            </div>
        </div>
    </body>;
}
