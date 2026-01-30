# Facilitator User Guide

This guide is a manual for **facilitators (instructors/organizers)** who use Open Codelabs to run hands-on sessions.

---

## 1. Before You Start

### Prerequisites
- Open Codelabs system installed ([Installation Guide](../getting-started/installation.md))
- Hands-on content (in Markdown format)
- Session environment (local or public server)

### Facilitator Roles
- Create and manage Codelabs
- Operate sessions and monitor progress
- Verify attendee progress
- Collect real-time feedback

---

## 2. Accessing the System

### Start in Local Environment
```bash
# Start with Docker Compose
docker compose up --build
```

- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:8080

### Accessing the Admin Page
1. Access `http://localhost:5173` in your browser.
2. Click **"Admin"** from the top-left menu.
3. Enter the facilitator-specific management page.

---

## 3. Creating a Codelab

### 3-1. Creating a Codelab Manually

1. Click the **"Create New Codelab"** button on the **Admin page**.
2. Enter the following information:
   - **Title**: Codelab title (e.g., "Getting Started with Docker")
   - **Description**: A brief description.
   - **Duration**: Estimated time required (in minutes).
   - **Tags**: Related keywords (separated by commas).
3. Write **Markdown content**:
   ```markdown
   # Step 1: Environment Setup

   Let's install and verify Docker.

   ```bash
   docker --version
   ```

   # Step 2: Running Your First Container

   Let's run the Hello World container.

   ```bash
   docker run hello-world
   ```
   ```
4. Click the **"Save"** button to save.

### 3-2. Auto-generating Codelabs with AI (Gemini Integration)

1. Click **"AI Codelab Generator"** on the Admin page.
2. Enter the desired topic (e.g., "Kubernetes Basics").
3. Select **Normal Mode** or **Pro Mode** in Generation Mode.
   - **Normal Mode**: Fast generation, quick result verification.
   - **Pro Mode**: Proceeds through stages (Plan → Draft → Review → Reflect), allowing you to check step-by-step results and changes (diff).
4. The AI automatically generates step-by-step content.
5. Review the generated content and edit if necessary.
6. Click the **"Save"** button to save.

!!! tip "AI Generation Tips"
    - Specifying a detailed topic and target level will result in more accurate content.
    - Example: "Introduction to Python Flask Web Development for Beginners"

---

## 4. Managing Codelabs

### Viewing Codelab List
- Verify all created Codelabs on the Admin page.
- Check the status, number of participants, and completion rate of each Codelab.

### Editing a Codelab
1. Select the Codelab to edit on the Admin page.
2. Click the **"Edit"** button.
3. Edit the content and click **"Update"** to save.

### Deleting a Codelab
1. Select the Codelab to delete on the Admin page.
2. Click the **"Delete"** button.
3. Confirm and delete (cannot be recovered).

---

## 5. Running a Session

### 5-1. Sharing Access Information with Participants

#### When running on a local network:
```
Access URL: http://YOUR_LOCAL_IP:5173
```

#### When external access is required (using ngrok/bore/cloudflare):
```bash
./run-public.sh --ngrok
# or
./run-public.sh --bore
# or
./run-public.sh --cloudflare
```

After running the script:
- A **QR code** is automatically generated.
- Share the QR code with participants by displaying it on a screen.
- Or directly share the generated Public URL.

!!! warning "Security Caution"
    Be sure to terminate the Public URL after the session ends.

### 5-2. Monitoring Participants

Check in real-time on the Admin page:
- **Number of Participants**: Attendees currently connected.
- **Progress**: Percentage of participants who have completed each step.
- **Feedback**: Questions and feedback left by participants.

### 5-3. Providing Real-time Help

When a participant is stuck on a specific step:
1. Check the progress on the Admin dashboard.
2. If the completion rate for that step is low, provide additional explanation.
3. If necessary, immediately edit the step content to improve it.

---

## 6. Verifying Progress

### Information Available on the Dashboard
- **Total number of participants**
- **Completion rate for each step** (Progress %)
- **Average time spent**
- **Feedback and question history**

### Verifying Individual Participant Progress
1. Click **"Participants"** on the Admin page.
2. Check each participant's current progress step.
3. Identify and support participants who are progressing slowly.

---

## 7. Collecting and Analyzing Feedback

### Checking Real-time Feedback
- Check in the **"Feedback"** section on the Admin page.
- See opinions and questions left by participants at each step.

### After the Session
- Verify the overall completion rate.
- Analyze difficult steps (steps with low completion rates).
- Use as material for improving the next session.

---

## 8. Tips and Best Practices

### Before the Session
- [ ] Test the Codelab content in advance.
- [ ] Set realistic estimated time requirements.
- [ ] Announce necessary prerequisites (software, accounts, etc.) in advance.

### During the Session
- [ ] Go through the first step together to guide them on how to use the system.
- [ ] Periodically check progress and adjust pace if necessary.
- [ ] Actively take questions and utilize real-time feedback.

### After the Session
- [ ] Provide completion certificates or materials to participants.
- [ ] Analyze feedback to improve the next session.
- [ ] Update Codelab content.

---

## 9. Troubleshooting

### Participants Cannot Access
- Check firewall settings.
- Check if the Public URL was generated correctly.
- Check the network connection status.

### System is Slow
- Check if there are too many concurrent users.
- Check server resources (CPU, Memory).
- Attempt to restart Docker containers.

### Content is Not Displaying Correctly
- Check for Markdown syntax errors.
- Check the language specification for code blocks (e.g., ```bash, ```python).
- Re-access after clearing browser cache.

---

## 10. Additional Resources

- [Creating Your First Codelab](../getting-started/first-codelab.md)
- [API Reference](../specification/api-reference.md)
- [FAQ](../faq.md)
- [Public Deployment Guide](../self-hosting/public-deployment.md)

---

!!! success "Ready to go!"
    Now you are ready to run a successful hands-on session! 🎉
