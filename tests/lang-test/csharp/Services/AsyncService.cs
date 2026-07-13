// ripex-lang-test: C# async service — async/await, Task, cancellation.
using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Ripex.Services
{
    public class AsyncService
    {
        public async Task<string> FetchJsonAsync(string url)
        {
            await Task.Delay(0);
            return url;
        }

        public async Task<List<string>> FetchAllAsync(IEnumerable<string> urls)
        {
            var tasks = new List<Task<string>>();
            foreach (var u in urls)
            {
                tasks.Add(FetchJsonAsync(u));
            }
            var results = await Task.WhenAll(tasks);
            return new List<string>(results);
        }

        public string WithTimeout(CancellationToken token)
        {
            return token.IsCancellationRequested ? "cancelled" : "ok";
        }
    }
}
