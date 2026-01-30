# License

Open Codelabs is distributed under the Apache License 2.0.

## Apache License 2.0

```
Copyright 2026 JAICHANGPARK

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## Summary

Apache License 2.0 is a **permissive open-source license** that allows:

### ✅ Permitted

- **Commercial Use**: Can be used for commercial purposes.
- **Modification**: Can modify the source code.
- **Distribution**: Can distribute the original or modified version.
- **Patent Use**: Can use the contributors' patents.
- **Private Use**: No obligation to make it public.

### 📋 Conditions

- **License and Copyright Notice**: Must include the original license and copyright notice.
- **State Changes**: Changes to files must be stated.
- **Same License Usage**: Maintain the Apache 2.0 license (for the original code parts).

### 🚫 Restrictions

- **Limitation of Liability**: No liability for any results of using the software.
- **No Warranty**: Provided "as is".
- **No Trademark Use**: Restrictions on using the project name/logo.

## Commercial Use

You can use Open Codelabs commercially:

### Possible Use Cases

1. **Workshops/Educational Services**
   - Use in paid workshops.
   - Integrate into corporate training programs.
   - Utilize in online lecture platforms.

2. **SaaS Offering**
   - Provide as a cloud service.
   - Operate under a subscription model.
   - Sell enterprise versions.

3. **Product Integration**
   - Integrate into your own products.
   - Customize and resell.
   - Use as an OEM product.

### Compliance Requirements

Even for commercial use, you must comply with the following:

1. **License Notice**
   ```
   This product includes Open Codelabs software
   developed by JAICHANGPARK

   Copyright 2026 JAICHANGPARK

   Licensed under the Apache License, Version 2.0
   ```

2. **NOTICE File** (if available)
   - Include the contents of the original NOTICE file.
   - State in product documentation or About page.

3. **State Modifications**
   - State change history in modified files.
   - Example: `// Modified by Company X on 2024-01-01`

## Difference from MIT License

While the README might mention the MIT License, the actual LICENSE file is Apache 2.0.

### Advantages of Apache 2.0

| Feature | MIT | Apache 2.0 |
|---------|-----|------------|
| Permission to use | ✅ | ✅ |
| Permission to modify | ✅ | ✅ |
| Commercial Use | ✅ | ✅ |
| **Patent Protection** | ❌ | ✅ |
| **State Modifications** | ❌ | ✅ (Explicit) |
| Simplicity | Very simple | Detailed |

**Why Apache 2.0 is better**:
- Explicit permission regarding patents.
- Enhanced contributor protection.
- Legal clarity.

## Dependency Licenses

Licenses for the main libraries used by Open Codelabs:

### Backend (Rust)

| Library | License | Description |
|---------|---------|-------------|
| Axum | MIT | Web Framework |
| Tokio | MIT | Async Runtime |
| SQLx | Apache-2.0 / MIT | Database |
| Serde | Apache-2.0 / MIT | Serialization |
| Tower | MIT | Middleware |

### Frontend (TypeScript)

| Library | License | Description |
|---------|---------|-------------|
| Svelte | MIT | UI Framework |
| SvelteKit | MIT | Full-stack Framework |
| Vite | MIT | Build Tool |
| Tailwind CSS | MIT | CSS Framework |
| marked | MIT | Markdown Parser |
| DOMPurify | Apache-2.0 / MPL-2.0 | XSS Prevention |

Since all dependencies use permissive licenses, there are **no issues with commercial use**.

## License for Contributions

When contributing to the project:

### Contribution Agreement

By contributing code:
- It is automatically licensed under Apache License 2.0.
- Contributors retain copyright.
- The project can use the contributed code under Apache 2.0.

### Including Other Code

When including code from other projects:

1. **Check Compatible Licenses**
   - MIT, BSD, Apache 2.0: ✅ Possible
   - GPL, AGPL: ❌ Not possible (copyleft)

2. **State Source**
   ```rust
   // This function is based on code from Project X
   // Copyright 2023 Original Author
   // Licensed under MIT License
   ```

3. **Add License File**
   - `licenses/PROJECT_NAME.txt`

## Frequently Asked Questions

### Q: Do I have to make the source code public?

**A:** No. Apache 2.0 does not require making the source code public. You can keep modified versions private.

### Q: Can I change the project name?

**A:** Yes, you can use a new name for the modified version. However, you must state that the original is "Open Codelabs".

### Q: Can I sell it for a fee?

**A:** Yes, absolutely. Apache 2.0 explicitly allows commercial use.

### Q: Am I protected from patent litigation?

**A:** Apache 2.0 includes an explicit grant of patent rights from contributors. However, if you initiate litigation, the patent license terminates.

### Q: Where should I put the license notice?

**A:** Any of the following:
- Product documentation
- README file
- About/Legal page
- Display within the software

## Full License Text

The full Apache License 2.0 text can be found in the [LICENSE](https://github.com/JAICHANGPARK/open-codelabs/blob/main/LICENSE) file in the project root.

## Additional Information

- [Official Apache License 2.0 Text](https://www.apache.org/licenses/LICENSE-2.0)
- [Apache License FAQ](https://www.apache.org/foundation/license-faq.html)
- [Choose a License](https://choosealicense.com/licenses/apache-2.0/)

## Contact

If you have any questions regarding the license:

- [GitHub Issues](https://github.com/JAICHANGPARK/open-codelabs/issues)
- [Email](mailto:team@example.com)
